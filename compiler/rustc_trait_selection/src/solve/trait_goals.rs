//! Dealing with trait goals, i.e. `T: Trait<'a, U>`.

use super::assembly::{self, structural_traits};
use super::search_graph::OverflowHandler;
use super::{EvalCtxt, SolverMode};
use rustc_hir::def_id::DefId;
use rustc_hir::{LangItem, Movability};
use rustc_infer::traits::query::NoSolution;
use rustc_middle::traits::solve::inspect::CandidateKind;
use rustc_middle::traits::solve::{CanonicalResponse, Certainty, Goal, MaybeCause, QueryResult};
use rustc_middle::traits::{BuiltinImplSource, Reveal};
use rustc_middle::ty::fast_reject::{DeepRejectCtxt, TreatParams, TreatProjections};
use rustc_middle::ty::{self, ToPredicate, Ty, TyCtxt};
use rustc_middle::ty::{TraitPredicate, TypeVisitableExt};
use rustc_span::{ErrorGuaranteed, DUMMY_SP};

impl<'tcx> assembly::GoalKind<'tcx> for TraitPredicate<'tcx> {
    fn self_ty(self) -> Ty<'tcx> {
        self.self_ty()
    }

    fn trait_ref(self, _: TyCtxt<'tcx>) -> ty::TraitRef<'tcx> {
        self.trait_ref
    }

    fn with_self_ty(self, tcx: TyCtxt<'tcx>, self_ty: Ty<'tcx>) -> Self {
        self.with_self_ty(tcx, self_ty)
    }

    fn trait_def_id(self, _: TyCtxt<'tcx>) -> DefId {
        self.def_id()
    }

    fn consider_impl_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, TraitPredicate<'tcx>>,
        impl_def_id: DefId,
    ) -> QueryResult<'tcx> {
        let tcx = ecx.tcx();

        let impl_trait_ref = tcx.impl_trait_ref(impl_def_id).unwrap();
        let drcx = DeepRejectCtxt { treat_obligation_params: TreatParams::ForLookup };
        if !drcx
            .args_refs_may_unify(goal.predicate.trait_ref.args, impl_trait_ref.skip_binder().args)
        {
            return Err(NoSolution);
        }

        let impl_polarity = tcx.impl_polarity(impl_def_id);
        // An upper bound of the certainty of this goal, used to lower the certainty
        // of reservation impl to ambiguous during coherence.
        let maximal_certainty = match impl_polarity {
            ty::ImplPolarity::Positive | ty::ImplPolarity::Negative => {
                match impl_polarity == goal.predicate.polarity {
                    true => Certainty::Yes,
                    false => return Err(NoSolution),
                }
            }
            ty::ImplPolarity::Reservation => match ecx.solver_mode() {
                SolverMode::Normal => return Err(NoSolution),
                SolverMode::Coherence => Certainty::AMBIGUOUS,
            },
        };

        ecx.probe_candidate("impl").enter(|ecx| {
            let impl_args = ecx.fresh_args_for_item(impl_def_id);
            let impl_trait_ref = impl_trait_ref.instantiate(tcx, impl_args);

            ecx.eq(goal.param_env, goal.predicate.trait_ref, impl_trait_ref)?;
            let where_clause_bounds = tcx
                .predicates_of(impl_def_id)
                .instantiate(tcx, impl_args)
                .predicates
                .into_iter()
                .map(|pred| goal.with(tcx, pred));
            ecx.add_goals(where_clause_bounds);

            ecx.evaluate_added_goals_and_make_canonical_response(maximal_certainty)
        })
    }

    fn consider_error_guaranteed_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        _guar: ErrorGuaranteed,
    ) -> QueryResult<'tcx> {
        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    fn probe_and_match_goal_against_assumption(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
        assumption: ty::Clause<'tcx>,
        then: impl FnOnce(&mut EvalCtxt<'_, 'tcx>) -> QueryResult<'tcx>,
    ) -> QueryResult<'tcx> {
        if let Some(trait_clause) = assumption.as_trait_clause() {
            if trait_clause.def_id() == goal.predicate.def_id()
                && trait_clause.polarity() == goal.predicate.polarity
            {
                // FIXME: Constness
                ecx.probe_candidate("assumption").enter(|ecx| {
                    let assumption_trait_pred = ecx.instantiate_binder_with_infer(trait_clause);
                    ecx.eq(
                        goal.param_env,
                        goal.predicate.trait_ref,
                        assumption_trait_pred.trait_ref,
                    )?;
                    then(ecx)
                })
            } else {
                Err(NoSolution)
            }
        } else {
            Err(NoSolution)
        }
    }

    fn consider_auto_trait_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        if let Some(result) = ecx.disqualify_auto_trait_candidate_due_to_possible_impl(goal) {
            return result;
        }

        // Don't call `type_of` on a local TAIT that's in the defining scope,
        // since that may require calling `typeck` on the same item we're
        // currently type checking, which will result in a fatal cycle that
        // ideally we want to avoid, since we can make progress on this goal
        // via an alias bound or a locally-inferred hidden type instead.
        //
        // Also, don't call `type_of` on a TAIT in `Reveal::All` mode, since
        // we already normalize the self type in
        // `assemble_candidates_after_normalizing_self_ty`, and we'd
        // just be registering an identical candidate here.
        //
        // Returning `Err(NoSolution)` here is ok in `SolverMode::Coherence`
        // since we'll always be registering an ambiguous candidate in
        // `assemble_candidates_after_normalizing_self_ty` due to normalizing
        // the TAIT.
        if let ty::Alias(ty::Opaque, opaque_ty) = goal.predicate.self_ty().kind() {
            if matches!(goal.param_env.reveal(), Reveal::All)
                || opaque_ty
                    .def_id
                    .as_local()
                    .is_some_and(|def_id| ecx.can_define_opaque_ty(def_id))
            {
                return Err(NoSolution);
            }
        }

        ecx.probe_and_evaluate_goal_for_constituent_tys(
            goal,
            structural_traits::instantiate_constituent_tys_for_auto_trait,
        )
    }

    fn consider_trait_alias_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        let tcx = ecx.tcx();

        ecx.probe_candidate("trait alias").enter(|ecx| {
            let nested_obligations = tcx
                .predicates_of(goal.predicate.def_id())
                .instantiate(tcx, goal.predicate.trait_ref.args);
            ecx.add_goals(nested_obligations.predicates.into_iter().map(|p| goal.with(tcx, p)));
            ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        })
    }

    fn consider_builtin_sized_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        ecx.probe_and_evaluate_goal_for_constituent_tys(
            goal,
            structural_traits::instantiate_constituent_tys_for_sized_trait,
        )
    }

    fn consider_builtin_copy_clone_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        ecx.probe_and_evaluate_goal_for_constituent_tys(
            goal,
            structural_traits::instantiate_constituent_tys_for_copy_clone_trait,
        )
    }

    fn consider_builtin_pointer_like_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        // The regions of a type don't affect the size of the type
        let tcx = ecx.tcx();
        // We should erase regions from both the param-env and type, since both
        // may have infer regions. Specifically, after canonicalizing and instantiating,
        // early bound regions turn into region vars in both the new and old solver.
        let key = tcx.erase_regions(goal.param_env.and(goal.predicate.self_ty()));
        // But if there are inference variables, we have to wait until it's resolved.
        if key.has_non_region_infer() {
            return ecx.evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS);
        }

        if let Ok(layout) = tcx.layout_of(key)
            && layout.layout.is_pointer_like(&tcx.data_layout)
        {
            // FIXME: We could make this faster by making a no-constraints response
            ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        } else {
            Err(NoSolution)
        }
    }

    fn consider_builtin_fn_ptr_trait_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        if let ty::FnPtr(..) = goal.predicate.self_ty().kind() {
            ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        } else {
            Err(NoSolution)
        }
    }

    fn consider_builtin_fn_trait_candidates(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
        goal_kind: ty::ClosureKind,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        let tcx = ecx.tcx();
        let tupled_inputs_and_output =
            match structural_traits::extract_tupled_inputs_and_output_from_callable(
                tcx,
                goal.predicate.self_ty(),
                goal_kind,
            )? {
                Some(a) => a,
                None => {
                    return ecx
                        .evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS);
                }
            };
        let output_is_sized_pred = tupled_inputs_and_output.map_bound(|(_, output)| {
            ty::TraitRef::from_lang_item(tcx, LangItem::Sized, DUMMY_SP, [output])
        });

        let pred = tupled_inputs_and_output
            .map_bound(|(inputs, _)| {
                ty::TraitRef::new(tcx, goal.predicate.def_id(), [goal.predicate.self_ty(), inputs])
            })
            .to_predicate(tcx);
        // A built-in `Fn` impl only holds if the output is sized.
        // (FIXME: technically we only need to check this if the type is a fn ptr...)
        Self::consider_implied_clause(ecx, goal, pred, [goal.with(tcx, output_is_sized_pred)])
    }

    fn consider_builtin_tuple_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        if let ty::Tuple(..) = goal.predicate.self_ty().kind() {
            ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        } else {
            Err(NoSolution)
        }
    }

    fn consider_builtin_pointee_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    fn consider_builtin_future_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        let ty::Generator(def_id, _, _) = *goal.predicate.self_ty().kind() else {
            return Err(NoSolution);
        };

        // Generators are not futures unless they come from `async` desugaring
        let tcx = ecx.tcx();
        if !tcx.generator_is_async(def_id) {
            return Err(NoSolution);
        }

        // Async generator unconditionally implement `Future`
        // Technically, we need to check that the future output type is Sized,
        // but that's already proven by the generator being WF.
        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    fn consider_builtin_generator_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        let self_ty = goal.predicate.self_ty();
        let ty::Generator(def_id, args, _) = *self_ty.kind() else {
            return Err(NoSolution);
        };

        // `async`-desugared generators do not implement the generator trait
        let tcx = ecx.tcx();
        if tcx.generator_is_async(def_id) {
            return Err(NoSolution);
        }

        let generator = args.as_generator();
        Self::consider_implied_clause(
            ecx,
            goal,
            ty::TraitRef::new(tcx, goal.predicate.def_id(), [self_ty, generator.resume_ty()])
                .to_predicate(tcx),
            // Technically, we need to check that the generator types are Sized,
            // but that's already proven by the generator being WF.
            [],
        )
    }

    fn consider_builtin_unsize_and_upcast_candidates(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> Vec<(CanonicalResponse<'tcx>, BuiltinImplSource)> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return vec![];
        }

        ecx.probe(|_| CandidateKind::DynUpcastingAssembly).enter(|ecx| {
            let a_ty = goal.predicate.self_ty();
            // We need to normalize the b_ty since it's matched structurally
            // in the other functions below.
            let b_ty = match ecx
                .normalize_non_self_ty(goal.predicate.trait_ref.args.type_at(1), goal.param_env)
            {
                Ok(Some(b_ty)) => {
                    // If we have a type var, then bail with ambiguity.
                    if b_ty.is_ty_var() {
                        return vec![(
                            ecx.evaluate_added_goals_and_make_canonical_response(
                                Certainty::AMBIGUOUS,
                            )
                            .unwrap(),
                            BuiltinImplSource::Misc,
                        )];
                    } else {
                        b_ty
                    }
                }
                Ok(None) => {
                    return vec![(
                        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Maybe(
                            MaybeCause::Overflow,
                        ))
                        .unwrap(),
                        BuiltinImplSource::Misc,
                    )];
                }
                Err(_) => return vec![],
            };

            let mut results = vec![];
            results.extend(ecx.consider_builtin_dyn_upcast_candidates(goal.param_env, a_ty, b_ty));
            results.extend(
                ecx.consider_builtin_unsize_candidate(goal.with(ecx.tcx(), (a_ty, b_ty)))
                    .into_iter()
                    .map(|resp| {
                        // If we're unsizing from tuple -> tuple, detect
                        let source =
                            if matches!((a_ty.kind(), b_ty.kind()), (ty::Tuple(..), ty::Tuple(..)))
                            {
                                BuiltinImplSource::TupleUnsizing
                            } else {
                                BuiltinImplSource::Misc
                            };
                        (resp, source)
                    }),
            );

            results
        })
    }

    fn consider_builtin_discriminant_kind_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        // `DiscriminantKind` is automatically implemented for every type.
        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    fn consider_builtin_destruct_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        // FIXME(-Ztrait-solver=next): Implement this when we get const working in the new solver

        // `Destruct` is automatically implemented for every type in
        // non-const environments.
        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    fn consider_builtin_transmute_candidate(
        ecx: &mut EvalCtxt<'_, 'tcx>,
        goal: Goal<'tcx, Self>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.polarity != ty::ImplPolarity::Positive {
            return Err(NoSolution);
        }

        // `rustc_transmute` does not have support for type or const params
        if goal.has_non_region_placeholders() {
            return Err(NoSolution);
        }

        // Erase regions because we compute layouts in `rustc_transmute`,
        // which will ICE for region vars.
        let args = ecx.tcx().erase_regions(goal.predicate.trait_ref.args);

        let Some(assume) =
            rustc_transmute::Assume::from_const(ecx.tcx(), goal.param_env, args.const_at(3))
        else {
            return Err(NoSolution);
        };

        let certainty = ecx.is_transmutable(
            rustc_transmute::Types { dst: args.type_at(0), src: args.type_at(1) },
            args.type_at(2),
            assume,
        )?;
        ecx.evaluate_added_goals_and_make_canonical_response(certainty)
    }
}

impl<'tcx> EvalCtxt<'_, 'tcx> {
    fn consider_builtin_unsize_candidate(
        &mut self,
        goal: Goal<'tcx, (Ty<'tcx>, Ty<'tcx>)>,
    ) -> QueryResult<'tcx> {
        let Goal { param_env, predicate: (a_ty, b_ty) } = goal;
        self.probe_candidate("builtin unsize").enter(|ecx| {
            let tcx = ecx.tcx();
            match (a_ty.kind(), b_ty.kind()) {
                (ty::Infer(ty::TyVar(_)), _) | (_, ty::Infer(ty::TyVar(_))) => {
                    bug!("unexpected type variable in unsize goal")
                }
                // Trait upcasting, or `dyn Trait + Auto + 'a` -> `dyn Trait + 'b`
                (&ty::Dynamic(_, _, ty::Dyn), &ty::Dynamic(_, _, ty::Dyn)) => {
                    // Dyn upcasting is handled separately, since due to upcasting,
                    // when there are two supertraits that differ by args, we
                    // may return more than one query response.
                    Err(NoSolution)
                }
                // `T` -> `dyn Trait` unsizing
                (_, &ty::Dynamic(data, region, ty::Dyn)) => {
                    // Can only unsize to an object-safe type
                    if data
                        .principal_def_id()
                        .is_some_and(|def_id| !tcx.check_is_object_safe(def_id))
                    {
                        return Err(NoSolution);
                    }

                    let Some(sized_def_id) = tcx.lang_items().sized_trait() else {
                        return Err(NoSolution);
                    };
                    // Check that the type implements all of the predicates of the def-id.
                    // (i.e. the principal, all of the associated types match, and any auto traits)
                    ecx.add_goals(
                        data.iter()
                            .map(|pred| Goal::new(tcx, param_env, pred.with_self_ty(tcx, a_ty))),
                    );
                    // The type must be Sized to be unsized.
                    ecx.add_goal(Goal::new(
                        tcx,
                        param_env,
                        ty::TraitRef::new(tcx, sized_def_id, [a_ty]),
                    ));
                    // The type must outlive the lifetime of the `dyn` we're unsizing into.
                    ecx.add_goal(Goal::new(
                        tcx,
                        param_env,
                        ty::Binder::dummy(ty::OutlivesPredicate(a_ty, region)),
                    ));
                    ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                }
                // `[T; n]` -> `[T]` unsizing
                (&ty::Array(a_elem_ty, ..), &ty::Slice(b_elem_ty)) => {
                    // We just require that the element type stays the same
                    ecx.eq(param_env, a_elem_ty, b_elem_ty)?;
                    ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                }
                // Struct unsizing `Struct<T>` -> `Struct<U>` where `T: Unsize<U>`
                (&ty::Adt(a_def, a_args), &ty::Adt(b_def, b_args))
                    if a_def.is_struct() && a_def.did() == b_def.did() =>
                {
                    let unsizing_params = tcx.unsizing_params_for_adt(a_def.did());
                    // We must be unsizing some type parameters. This also implies
                    // that the struct has a tail field.
                    if unsizing_params.is_empty() {
                        return Err(NoSolution);
                    }

                    let tail_field = a_def.non_enum_variant().tail();
                    let tail_field_ty = tcx.type_of(tail_field.did);

                    let a_tail_ty = tail_field_ty.instantiate(tcx, a_args);
                    let b_tail_ty = tail_field_ty.instantiate(tcx, b_args);

                    // Substitute just the unsizing params from B into A. The type after
                    // this substitution must be equal to B. This is so we don't unsize
                    // unrelated type parameters.
                    let new_a_args =
                        tcx.mk_args_from_iter(a_args.iter().enumerate().map(|(i, a)| {
                            if unsizing_params.contains(i as u32) { b_args[i] } else { a }
                        }));
                    let unsized_a_ty = Ty::new_adt(tcx, a_def, new_a_args);

                    // Finally, we require that `TailA: Unsize<TailB>` for the tail field
                    // types.
                    ecx.eq(param_env, unsized_a_ty, b_ty)?;
                    ecx.add_goal(Goal::new(
                        tcx,
                        param_env,
                        ty::TraitRef::new(
                            tcx,
                            tcx.lang_items().unsize_trait().unwrap(),
                            [a_tail_ty, b_tail_ty],
                        ),
                    ));
                    ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                }
                // Tuple unsizing `(.., T)` -> `(.., U)` where `T: Unsize<U>`
                (&ty::Tuple(a_tys), &ty::Tuple(b_tys))
                    if a_tys.len() == b_tys.len() && !a_tys.is_empty() =>
                {
                    let (a_last_ty, a_rest_tys) = a_tys.split_last().unwrap();
                    let b_last_ty = b_tys.last().unwrap();

                    // Substitute just the tail field of B., and require that they're equal.
                    let unsized_a_ty =
                        Ty::new_tup_from_iter(tcx, a_rest_tys.iter().chain([b_last_ty]).copied());
                    ecx.eq(param_env, unsized_a_ty, b_ty)?;

                    // Similar to ADTs, require that the rest of the fields are equal.
                    ecx.add_goal(Goal::new(
                        tcx,
                        param_env,
                        ty::TraitRef::new(
                            tcx,
                            tcx.lang_items().unsize_trait().unwrap(),
                            [*a_last_ty, *b_last_ty],
                        ),
                    ));
                    ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                }
                _ => Err(NoSolution),
            }
        })
    }

    fn consider_builtin_dyn_upcast_candidates(
        &mut self,
        param_env: ty::ParamEnv<'tcx>,
        a_ty: Ty<'tcx>,
        b_ty: Ty<'tcx>,
    ) -> Vec<(CanonicalResponse<'tcx>, BuiltinImplSource)> {
        if a_ty.is_ty_var() || b_ty.is_ty_var() {
            bug!("unexpected type variable in unsize goal")
        }

        let ty::Dynamic(a_data, a_region, ty::Dyn) = *a_ty.kind() else {
            return vec![];
        };
        let ty::Dynamic(b_data, b_region, ty::Dyn) = *b_ty.kind() else {
            return vec![];
        };

        let tcx = self.tcx();
        // All of a's auto traits need to be in b's auto traits.
        let auto_traits_compatible =
            b_data.auto_traits().all(|b| a_data.auto_traits().any(|a| a == b));
        if !auto_traits_compatible {
            return vec![];
        }

        // Try to match `a_ty` against `b_ty`, replacing `a_ty`'s principal trait ref with
        // the supertrait principal and subtyping the types.
        let unsize_dyn_to_principal =
            |ecx: &mut Self, principal: Option<ty::PolyExistentialTraitRef<'tcx>>| {
                ecx.probe_candidate("upcast dyn to principle").enter(
                    |ecx| -> Result<_, NoSolution> {
                        // Require that all of the trait predicates from A match B, except for
                        // the auto traits. We do this by constructing a new A type with B's
                        // auto traits, and equating these types.
                        let new_a_data = principal
                            .into_iter()
                            .map(|trait_ref| trait_ref.map_bound(ty::ExistentialPredicate::Trait))
                            .chain(a_data.iter().filter(|a| {
                                matches!(a.skip_binder(), ty::ExistentialPredicate::Projection(_))
                            }))
                            .chain(
                                b_data
                                    .auto_traits()
                                    .map(ty::ExistentialPredicate::AutoTrait)
                                    .map(ty::Binder::dummy),
                            );
                        let new_a_data = tcx.mk_poly_existential_predicates_from_iter(new_a_data);
                        let new_a_ty = Ty::new_dynamic(tcx, new_a_data, b_region, ty::Dyn);

                        // We also require that A's lifetime outlives B's lifetime.
                        ecx.eq(param_env, new_a_ty, b_ty)?;
                        ecx.add_goal(Goal::new(
                            tcx,
                            param_env,
                            ty::Binder::dummy(ty::OutlivesPredicate(a_region, b_region)),
                        ));
                        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                    },
                )
            };

        let mut responses = vec![];
        // If the principal def ids match (or are both none), then we're not doing
        // trait upcasting. We're just removing auto traits (or shortening the lifetime).
        if a_data.principal_def_id() == b_data.principal_def_id() {
            if let Ok(resp) = unsize_dyn_to_principal(self, a_data.principal()) {
                responses.push((resp, BuiltinImplSource::Misc));
            }
        } else if let Some(a_principal) = a_data.principal() {
            self.walk_vtable(
                a_principal.with_self_ty(tcx, a_ty),
                |ecx, new_a_principal, _, vtable_vptr_slot| {
                    if let Ok(resp) = unsize_dyn_to_principal(
                        ecx,
                        Some(new_a_principal.map_bound(|trait_ref| {
                            ty::ExistentialTraitRef::erase_self_ty(tcx, trait_ref)
                        })),
                    ) {
                        responses
                            .push((resp, BuiltinImplSource::TraitUpcasting { vtable_vptr_slot }));
                    }
                },
            );
        }

        responses
    }

    // Return `Some` if there is an impl (built-in or user provided) that may
    // hold for the self type of the goal, which for coherence and soundness
    // purposes must disqualify the built-in auto impl assembled by considering
    // the type's constituent types.
    fn disqualify_auto_trait_candidate_due_to_possible_impl(
        &mut self,
        goal: Goal<'tcx, TraitPredicate<'tcx>>,
    ) -> Option<QueryResult<'tcx>> {
        let self_ty = goal.predicate.self_ty();
        match *self_ty.kind() {
            // Stall int and float vars until they are resolved to a concrete
            // numerical type. That's because the check for impls below treats
            // int vars as matching any impl. Even if we filtered such impls,
            // we probably don't want to treat an `impl !AutoTrait for i32` as
            // disqualifying the built-in auto impl for `i64: AutoTrait` either.
            ty::Infer(ty::IntVar(_) | ty::FloatVar(_)) => {
                Some(self.evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS))
            }

            // These types cannot be structurally decomposed into constituent
            // types, and therefore have no built-in auto impl.
            ty::Dynamic(..)
            | ty::Param(..)
            | ty::Foreign(..)
            | ty::Alias(ty::Projection | ty::Weak | ty::Inherent, ..)
            | ty::Placeholder(..) => Some(Err(NoSolution)),

            ty::Infer(_) | ty::Bound(_, _) => bug!("unexpected type `{self_ty}`"),

            // Generators have one special built-in candidate, `Unpin`, which
            // takes precedence over the structural auto trait candidate being
            // assembled.
            ty::Generator(_, _, movability)
                if Some(goal.predicate.def_id()) == self.tcx().lang_items().unpin_trait() =>
            {
                match movability {
                    Movability::Static => Some(Err(NoSolution)),
                    Movability::Movable => {
                        Some(self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes))
                    }
                }
            }

            // For rigid types, any possible implementation that could apply to
            // the type (even if after unification and processing nested goals
            // it does not hold) will disqualify the built-in auto impl.
            //
            // This differs from the current stable behavior and fixes #84857.
            // Due to breakage found via crater, we currently instead lint
            // patterns which can be used to exploit this unsoundness on stable,
            // see #93367 for more details.
            ty::Bool
            | ty::Char
            | ty::Int(_)
            | ty::Uint(_)
            | ty::Float(_)
            | ty::Str
            | ty::Array(_, _)
            | ty::Slice(_)
            | ty::RawPtr(_)
            | ty::Ref(_, _, _)
            | ty::FnDef(_, _)
            | ty::FnPtr(_)
            | ty::Closure(_, _)
            | ty::Generator(_, _, _)
            | ty::GeneratorWitness(_)
            | ty::GeneratorWitnessMIR(_, _)
            | ty::Never
            | ty::Tuple(_)
            | ty::Adt(_, _)
            // FIXME: Handling opaques here is kinda sus. Especially because we
            // simplify them to SimplifiedType::Placeholder.
            | ty::Alias(ty::Opaque, _) => {
                let mut disqualifying_impl = None;
                self.tcx().for_each_relevant_impl_treating_projections(
                    goal.predicate.def_id(),
                    goal.predicate.self_ty(),
                    TreatProjections::NextSolverLookup,
                    |impl_def_id| {
                        disqualifying_impl = Some(impl_def_id);
                    },
                );
                if let Some(def_id) = disqualifying_impl {
                    debug!(?def_id, ?goal, "disqualified auto-trait implementation");
                    // No need to actually consider the candidate here,
                    // since we do that in `consider_impl_candidate`.
                    return Some(Err(NoSolution));
                } else {
                    None
                }
            }
            ty::Error(_) => None,
        }
    }

    /// Convenience function for traits that are structural, i.e. that only
    /// have nested subgoals that only change the self type. Unlike other
    /// evaluate-like helpers, this does a probe, so it doesn't need to be
    /// wrapped in one.
    fn probe_and_evaluate_goal_for_constituent_tys(
        &mut self,
        goal: Goal<'tcx, TraitPredicate<'tcx>>,
        constituent_tys: impl Fn(&EvalCtxt<'_, 'tcx>, Ty<'tcx>) -> Result<Vec<Ty<'tcx>>, NoSolution>,
    ) -> QueryResult<'tcx> {
        self.probe_candidate("constituent tys").enter(|ecx| {
            ecx.add_goals(
                constituent_tys(ecx, goal.predicate.self_ty())?
                    .into_iter()
                    .map(|ty| {
                        goal.with(
                            ecx.tcx(),
                            ty::Binder::dummy(goal.predicate.with_self_ty(ecx.tcx(), ty)),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        })
    }

    #[instrument(level = "debug", skip(self))]
    pub(super) fn compute_trait_goal(
        &mut self,
        goal: Goal<'tcx, TraitPredicate<'tcx>>,
    ) -> QueryResult<'tcx> {
        let candidates = self.assemble_and_evaluate_candidates(goal);
        self.merge_candidates(candidates)
    }

    /// Normalize a non-self type when it is structually matched on when solving
    /// a built-in goal. This is handled already through `assemble_candidates_after_normalizing_self_ty`
    /// for the self type, but for other goals, additional normalization of other
    /// arguments may be needed to completely implement the semantics of the trait.
    ///
    /// This is required when structurally matching on any trait argument that is
    /// not the self type.
    fn normalize_non_self_ty(
        &mut self,
        mut ty: Ty<'tcx>,
        param_env: ty::ParamEnv<'tcx>,
    ) -> Result<Option<Ty<'tcx>>, NoSolution> {
        if !matches!(ty.kind(), ty::Alias(..)) {
            return Ok(Some(ty));
        }

        self.repeat_while_none(
            |_| Ok(None),
            |ecx| {
                let ty::Alias(_, projection_ty) = *ty.kind() else {
                    return Some(Ok(Some(ty)));
                };

                let normalized_ty = ecx.next_ty_infer();
                let normalizes_to_goal = Goal::new(
                    ecx.tcx(),
                    param_env,
                    ty::Binder::dummy(ty::ProjectionPredicate {
                        projection_ty,
                        term: normalized_ty.into(),
                    }),
                );
                ecx.add_goal(normalizes_to_goal);
                if let Err(err) = ecx.try_evaluate_added_goals() {
                    return Some(Err(err));
                }

                ty = ecx.resolve_vars_if_possible(normalized_ty);
                None
            },
        )
    }
}
