use super::potentially_plural_count;
use crate::errors::LifetimesOrBoundsMismatchOnTrait;
use hir::def_id::{DefId, LocalDefId};
use rustc_data_structures::fx::{FxHashMap, FxIndexSet};
use rustc_errors::{
    pluralize, struct_span_err, Applicability, DiagnosticId, ErrorGuaranteed, MultiSpan,
};
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit;
use rustc_hir::{GenericParamKind, ImplItemKind};
use rustc_infer::infer::outlives::env::OutlivesEnvironment;
use rustc_infer::infer::type_variable::{TypeVariableOrigin, TypeVariableOriginKind};
use rustc_infer::infer::{self, InferCtxt, TyCtxtInferExt};
use rustc_infer::traits::util;
use rustc_middle::ty::error::{ExpectedFound, TypeError};
use rustc_middle::ty::util::ExplicitSelf;
use rustc_middle::ty::{
    self, GenericArgs, Ty, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt,
};
use rustc_middle::ty::{GenericParamDefKind, ToPredicate, TyCtxt};
use rustc_span::{Span, DUMMY_SP};
use rustc_trait_selection::traits::error_reporting::TypeErrCtxtExt;
use rustc_trait_selection::traits::outlives_bounds::InferCtxtExt as _;
use rustc_trait_selection::traits::{
    self, ObligationCause, ObligationCauseCode, ObligationCtxt, Reveal,
};
use std::borrow::Cow;
use std::iter;

/// Checks that a method from an impl conforms to the signature of
/// the same method as declared in the trait.
///
/// # Parameters
///
/// - `impl_m`: type of the method we are checking
/// - `trait_m`: the method in the trait
/// - `impl_trait_ref`: the TraitRef corresponding to the trait implementation
pub(super) fn compare_impl_method<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
) {
    debug!("compare_impl_method(impl_trait_ref={:?})", impl_trait_ref);

    let _: Result<_, ErrorGuaranteed> = try {
        check_method_is_structurally_compatible(tcx, impl_m, trait_m, impl_trait_ref, false)?;
        compare_method_predicate_entailment(
            tcx,
            impl_m,
            trait_m,
            impl_trait_ref,
            CheckImpliedWfMode::Check,
        )?;
    };
}

/// Checks a bunch of different properties of the impl/trait methods for
/// compatibility, such as asyncness, number of argument, self receiver kind,
/// and number of early- and late-bound generics.
fn check_method_is_structurally_compatible<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    compare_self_type(tcx, impl_m, trait_m, impl_trait_ref, delay)?;
    compare_number_of_generics(tcx, impl_m, trait_m, delay)?;
    compare_generic_param_kinds(tcx, impl_m, trait_m, delay)?;
    compare_number_of_method_arguments(tcx, impl_m, trait_m, delay)?;
    compare_synthetic_generics(tcx, impl_m, trait_m, delay)?;
    compare_asyncness(tcx, impl_m, trait_m, delay)?;
    check_region_bounds_on_impl_item(tcx, impl_m, trait_m, delay)?;
    Ok(())
}

/// This function is best explained by example. Consider a trait with it's implementation:
///
/// ```rust
/// trait Trait<'t, T> {
///     // `trait_m`
///     fn method<'a, M>(t: &'t T, m: &'a M) -> Self;
/// }
///
/// struct Foo;
///
/// impl<'i, 'j, U> Trait<'j, &'i U> for Foo {
///     // `impl_m`
///     fn method<'b, N>(t: &'j &'i U, m: &'b N) -> Foo { Foo }
/// }
/// ```
///
/// We wish to decide if those two method types are compatible.
/// For this we have to show that, assuming the bounds of the impl hold, the
/// bounds of `trait_m` imply the bounds of `impl_m`.
///
/// We start out with `trait_to_impl_args`, that maps the trait
/// type parameters to impl type parameters. This is taken from the
/// impl trait reference:
///
/// ```rust,ignore (pseudo-Rust)
/// trait_to_impl_args = {'t => 'j, T => &'i U, Self => Foo}
/// ```
///
/// We create a mapping `dummy_args` that maps from the impl type
/// parameters to fresh types and regions. For type parameters,
/// this is the identity transform, but we could as well use any
/// placeholder types. For regions, we convert from bound to free
/// regions (Note: but only early-bound regions, i.e., those
/// declared on the impl or used in type parameter bounds).
///
/// ```rust,ignore (pseudo-Rust)
/// impl_to_placeholder_args = {'i => 'i0, U => U0, N => N0 }
/// ```
///
/// Now we can apply `placeholder_args` to the type of the impl method
/// to yield a new function type in terms of our fresh, placeholder
/// types:
///
/// ```rust,ignore (pseudo-Rust)
/// <'b> fn(t: &'i0 U0, m: &'b) -> Foo
/// ```
///
/// We now want to extract and substitute the type of the *trait*
/// method and compare it. To do so, we must create a compound
/// substitution by combining `trait_to_impl_args` and
/// `impl_to_placeholder_args`, and also adding a mapping for the method
/// type parameters. We extend the mapping to also include
/// the method parameters.
///
/// ```rust,ignore (pseudo-Rust)
/// trait_to_placeholder_args = { T => &'i0 U0, Self => Foo, M => N0 }
/// ```
///
/// Applying this to the trait method type yields:
///
/// ```rust,ignore (pseudo-Rust)
/// <'a> fn(t: &'i0 U0, m: &'a) -> Foo
/// ```
///
/// This type is also the same but the name of the bound region (`'a`
/// vs `'b`). However, the normal subtyping rules on fn types handle
/// this kind of equivalency just fine.
///
/// We now use these substitutions to ensure that all declared bounds are
/// satisfied by the implementation's method.
///
/// We do this by creating a parameter environment which contains a
/// substitution corresponding to `impl_to_placeholder_args`. We then build
/// `trait_to_placeholder_args` and use it to convert the predicates contained
/// in the `trait_m` generics to the placeholder form.
///
/// Finally we register each of these predicates as an obligation and check that
/// they hold.
#[instrument(level = "debug", skip(tcx, impl_trait_ref))]
fn compare_method_predicate_entailment<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
    check_implied_wf: CheckImpliedWfMode,
) -> Result<(), ErrorGuaranteed> {
    let trait_to_impl_args = impl_trait_ref.args;

    // This node-id should be used for the `body_id` field on each
    // `ObligationCause` (and the `FnCtxt`).
    //
    // FIXME(@lcnr): remove that after removing `cause.body_id` from
    // obligations.
    let impl_m_def_id = impl_m.def_id.expect_local();
    let impl_m_span = tcx.def_span(impl_m_def_id);
    let cause = ObligationCause::new(
        impl_m_span,
        impl_m_def_id,
        ObligationCauseCode::CompareImplItemObligation {
            impl_item_def_id: impl_m_def_id,
            trait_item_def_id: trait_m.def_id,
            kind: impl_m.kind,
        },
    );

    // Create mapping from impl to placeholder.
    let impl_to_placeholder_args = GenericArgs::identity_for_item(tcx, impl_m.def_id);

    // Create mapping from trait to placeholder.
    let trait_to_placeholder_args =
        impl_to_placeholder_args.rebase_onto(tcx, impl_m.container_id(tcx), trait_to_impl_args);
    debug!("compare_impl_method: trait_to_placeholder_args={:?}", trait_to_placeholder_args);

    let impl_m_predicates = tcx.predicates_of(impl_m.def_id);
    let trait_m_predicates = tcx.predicates_of(trait_m.def_id);

    // Create obligations for each predicate declared by the impl
    // definition in the context of the trait's parameter
    // environment. We can't just use `impl_env.caller_bounds`,
    // however, because we want to replace all late-bound regions with
    // region variables.
    let impl_predicates = tcx.predicates_of(impl_m_predicates.parent.unwrap());
    let mut hybrid_preds = impl_predicates.instantiate_identity(tcx);

    debug!("compare_impl_method: impl_bounds={:?}", hybrid_preds);

    // This is the only tricky bit of the new way we check implementation methods
    // We need to build a set of predicates where only the method-level bounds
    // are from the trait and we assume all other bounds from the implementation
    // to be previously satisfied.
    //
    // We then register the obligations from the impl_m and check to see
    // if all constraints hold.
    hybrid_preds.predicates.extend(
        trait_m_predicates
            .instantiate_own(tcx, trait_to_placeholder_args)
            .map(|(predicate, _)| predicate),
    );

    // Construct trait parameter environment and then shift it into the placeholder viewpoint.
    // The key step here is to update the caller_bounds's predicates to be
    // the new hybrid bounds we computed.
    let normalize_cause = traits::ObligationCause::misc(impl_m_span, impl_m_def_id);
    let param_env = ty::ParamEnv::new(tcx.mk_clauses(&hybrid_preds.predicates), Reveal::UserFacing);
    let param_env = traits::normalize_param_env_or_error(tcx, param_env, normalize_cause);

    let infcx = &tcx.infer_ctxt().build();
    let ocx = ObligationCtxt::new(infcx);

    debug!("compare_impl_method: caller_bounds={:?}", param_env.caller_bounds());

    let impl_m_own_bounds = impl_m_predicates.instantiate_own(tcx, impl_to_placeholder_args);
    for (predicate, span) in impl_m_own_bounds {
        let normalize_cause = traits::ObligationCause::misc(span, impl_m_def_id);
        let predicate = ocx.normalize(&normalize_cause, param_env, predicate);

        let cause = ObligationCause::new(
            span,
            impl_m_def_id,
            ObligationCauseCode::CompareImplItemObligation {
                impl_item_def_id: impl_m_def_id,
                trait_item_def_id: trait_m.def_id,
                kind: impl_m.kind,
            },
        );
        ocx.register_obligation(traits::Obligation::new(tcx, cause, param_env, predicate));
    }

    // We now need to check that the signature of the impl method is
    // compatible with that of the trait method. We do this by
    // checking that `impl_fty <: trait_fty`.
    //
    // FIXME. Unfortunately, this doesn't quite work right now because
    // associated type normalization is not integrated into subtype
    // checks. For the comparison to be valid, we need to
    // normalize the associated types in the impl/trait methods
    // first. However, because function types bind regions, just
    // calling `normalize_associated_types_in` would have no effect on
    // any associated types appearing in the fn arguments or return
    // type.

    // Compute placeholder form of impl and trait method tys.
    let tcx = infcx.tcx;

    let mut wf_tys = FxIndexSet::default();

    let unnormalized_impl_sig = infcx.instantiate_binder_with_fresh_vars(
        impl_m_span,
        infer::HigherRankedType,
        tcx.fn_sig(impl_m.def_id).instantiate_identity(),
    );
    let unnormalized_impl_fty = Ty::new_fn_ptr(tcx, ty::Binder::dummy(unnormalized_impl_sig));

    let norm_cause = ObligationCause::misc(impl_m_span, impl_m_def_id);
    let impl_sig = ocx.normalize(&norm_cause, param_env, unnormalized_impl_sig);
    debug!("compare_impl_method: impl_fty={:?}", impl_sig);

    let trait_sig = tcx.fn_sig(trait_m.def_id).instantiate(tcx, trait_to_placeholder_args);
    let trait_sig = tcx.liberate_late_bound_regions(impl_m.def_id, trait_sig);

    // Next, add all inputs and output as well-formed tys. Importantly,
    // we have to do this before normalization, since the normalized ty may
    // not contain the input parameters. See issue #87748.
    wf_tys.extend(trait_sig.inputs_and_output.iter());
    let trait_sig = ocx.normalize(&norm_cause, param_env, trait_sig);
    // We also have to add the normalized trait signature
    // as we don't normalize during implied bounds computation.
    wf_tys.extend(trait_sig.inputs_and_output.iter());
    let trait_fty = Ty::new_fn_ptr(tcx, ty::Binder::dummy(trait_sig));

    debug!("compare_impl_method: trait_fty={:?}", trait_fty);

    // FIXME: We'd want to keep more accurate spans than "the method signature" when
    // processing the comparison between the trait and impl fn, but we sadly lose them
    // and point at the whole signature when a trait bound or specific input or output
    // type would be more appropriate. In other places we have a `Vec<Span>`
    // corresponding to their `Vec<Predicate>`, but we don't have that here.
    // Fixing this would improve the output of test `issue-83765.rs`.
    let result = ocx.sup(&cause, param_env, trait_sig, impl_sig);

    if let Err(terr) = result {
        debug!(?impl_sig, ?trait_sig, ?terr, "sub_types failed");

        let emitted = report_trait_method_mismatch(
            &infcx,
            cause,
            terr,
            (trait_m, trait_sig),
            (impl_m, impl_sig),
            impl_trait_ref,
        );
        return Err(emitted);
    }

    if check_implied_wf == CheckImpliedWfMode::Check && !(impl_sig, trait_sig).references_error() {
        // We need to check that the impl's args are well-formed given
        // the hybrid param-env (impl + trait method where-clauses).
        ocx.register_obligation(traits::Obligation::new(
            infcx.tcx,
            ObligationCause::dummy(),
            param_env,
            ty::Binder::dummy(ty::PredicateKind::Clause(ty::ClauseKind::WellFormed(
                unnormalized_impl_fty.into(),
            ))),
        ));
    }

    // Check that all obligations are satisfied by the implementation's
    // version.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        match check_implied_wf {
            CheckImpliedWfMode::Check => {
                let impl_m_hir_id = tcx.hir().local_def_id_to_hir_id(impl_m_def_id);
                return compare_method_predicate_entailment(
                    tcx,
                    impl_m,
                    trait_m,
                    impl_trait_ref,
                    CheckImpliedWfMode::Skip,
                )
                .map(|()| {
                    // If the skip-mode was successful, emit a lint.
                    emit_implied_wf_lint(infcx.tcx, impl_m, impl_m_hir_id, vec![]);
                });
            }
            CheckImpliedWfMode::Skip => {
                let reported = infcx.err_ctxt().report_fulfillment_errors(&errors);
                return Err(reported);
            }
        }
    }

    // Finally, resolve all regions. This catches wily misuses of
    // lifetime parameters.
    let outlives_env = OutlivesEnvironment::with_bounds(
        param_env,
        infcx.implied_bounds_tys(param_env, impl_m_def_id, wf_tys.clone()),
    );
    let errors = infcx.resolve_regions(&outlives_env);
    if !errors.is_empty() {
        // FIXME(compiler-errors): This can be simplified when IMPLIED_BOUNDS_ENTAILMENT
        // becomes a hard error (i.e. ideally we'd just call `resolve_regions_and_report_errors`
        let impl_m_hir_id = tcx.hir().local_def_id_to_hir_id(impl_m_def_id);
        match check_implied_wf {
            CheckImpliedWfMode::Check => {
                return compare_method_predicate_entailment(
                    tcx,
                    impl_m,
                    trait_m,
                    impl_trait_ref,
                    CheckImpliedWfMode::Skip,
                )
                .map(|()| {
                    let bad_args = extract_bad_args_for_implies_lint(
                        tcx,
                        &errors,
                        (trait_m, trait_sig),
                        // Unnormalized impl sig corresponds to the HIR types written
                        (impl_m, unnormalized_impl_sig),
                        impl_m_hir_id,
                    );
                    // If the skip-mode was successful, emit a lint.
                    emit_implied_wf_lint(tcx, impl_m, impl_m_hir_id, bad_args);
                });
            }
            CheckImpliedWfMode::Skip => {
                if infcx.tainted_by_errors().is_none() {
                    infcx.err_ctxt().report_region_errors(impl_m_def_id, &errors);
                }
                return Err(tcx
                    .sess
                    .delay_span_bug(rustc_span::DUMMY_SP, "error should have been emitted"));
            }
        }
    }

    Ok(())
}

fn extract_bad_args_for_implies_lint<'tcx>(
    tcx: TyCtxt<'tcx>,
    errors: &[infer::RegionResolutionError<'tcx>],
    (trait_m, trait_sig): (ty::AssocItem, ty::FnSig<'tcx>),
    (impl_m, impl_sig): (ty::AssocItem, ty::FnSig<'tcx>),
    hir_id: hir::HirId,
) -> Vec<(Span, Option<String>)> {
    let mut blame_generics = vec![];
    for error in errors {
        // Look for the subregion origin that contains an input/output type
        let origin = match error {
            infer::RegionResolutionError::ConcreteFailure(o, ..) => o,
            infer::RegionResolutionError::GenericBoundFailure(o, ..) => o,
            infer::RegionResolutionError::SubSupConflict(_, _, o, ..) => o,
            infer::RegionResolutionError::UpperBoundUniverseConflict(.., o, _) => o,
        };
        // Extract (possible) input/output types from origin
        match origin {
            infer::SubregionOrigin::Subtype(trace) => {
                if let Some((a, b)) = trace.values.ty() {
                    blame_generics.extend([a, b]);
                }
            }
            infer::SubregionOrigin::RelateParamBound(_, ty, _) => blame_generics.push(*ty),
            infer::SubregionOrigin::ReferenceOutlivesReferent(ty, _) => blame_generics.push(*ty),
            _ => {}
        }
    }

    let fn_decl = tcx.hir().fn_decl_by_hir_id(hir_id).unwrap();
    let opt_ret_ty = match fn_decl.output {
        hir::FnRetTy::DefaultReturn(_) => None,
        hir::FnRetTy::Return(ty) => Some(ty),
    };

    // Map late-bound regions from trait to impl, so the names are right.
    let mapping = std::iter::zip(
        tcx.fn_sig(trait_m.def_id).skip_binder().bound_vars(),
        tcx.fn_sig(impl_m.def_id).skip_binder().bound_vars(),
    )
    .filter_map(|(impl_bv, trait_bv)| {
        if let ty::BoundVariableKind::Region(impl_bv) = impl_bv
            && let ty::BoundVariableKind::Region(trait_bv) = trait_bv
        {
            Some((impl_bv, trait_bv))
        } else {
            None
        }
    })
    .collect();

    // For each arg, see if it was in the "blame" of any of the region errors.
    // If so, then try to produce a suggestion to replace the argument type with
    // one from the trait.
    let mut bad_args = vec![];
    for (idx, (ty, hir_ty)) in
        std::iter::zip(impl_sig.inputs_and_output, fn_decl.inputs.iter().chain(opt_ret_ty))
            .enumerate()
    {
        let expected_ty = trait_sig.inputs_and_output[idx]
            .fold_with(&mut RemapLateBound { tcx, mapping: &mapping });
        if blame_generics.iter().any(|blame| ty.contains(*blame)) {
            let expected_ty_sugg = expected_ty.to_string();
            bad_args.push((
                hir_ty.span,
                // Only suggest something if it actually changed.
                (expected_ty_sugg != ty.to_string()).then_some(expected_ty_sugg),
            ));
        }
    }

    bad_args
}

struct RemapLateBound<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    mapping: &'a FxHashMap<ty::BoundRegionKind, ty::BoundRegionKind>,
}

impl<'tcx> TypeFolder<TyCtxt<'tcx>> for RemapLateBound<'_, 'tcx> {
    fn interner(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn fold_region(&mut self, r: ty::Region<'tcx>) -> ty::Region<'tcx> {
        if let ty::ReFree(fr) = *r {
            ty::Region::new_free(
                self.tcx,
                fr.scope,
                self.mapping.get(&fr.bound_region).copied().unwrap_or(fr.bound_region),
            )
        } else {
            r
        }
    }
}

fn emit_implied_wf_lint<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    hir_id: hir::HirId,
    bad_args: Vec<(Span, Option<String>)>,
) {
    let span: MultiSpan = if bad_args.is_empty() {
        tcx.def_span(impl_m.def_id).into()
    } else {
        bad_args.iter().map(|(span, _)| *span).collect::<Vec<_>>().into()
    };
    tcx.struct_span_lint_hir(
        rustc_session::lint::builtin::IMPLIED_BOUNDS_ENTAILMENT,
        hir_id,
        span,
        "impl method assumes more implied bounds than the corresponding trait method",
        |lint| {
            let bad_args: Vec<_> =
                bad_args.into_iter().filter_map(|(span, sugg)| Some((span, sugg?))).collect();
            if !bad_args.is_empty() {
                lint.multipart_suggestion(
                    format!(
                        "replace {} type{} to make the impl signature compatible",
                        pluralize!("this", bad_args.len()),
                        pluralize!(bad_args.len())
                    ),
                    bad_args,
                    Applicability::MaybeIncorrect,
                );
            }
            lint
        },
    );
}

#[derive(Debug, PartialEq, Eq)]
enum CheckImpliedWfMode {
    /// Checks implied well-formedness of the impl method. If it fails, we will
    /// re-check with `Skip`, and emit a lint if it succeeds.
    Check,
    /// Skips checking implied well-formedness of the impl method, but will emit
    /// a lint if the `compare_method_predicate_entailment` succeeded. This means that
    /// the reason that we had failed earlier during `Check` was due to the impl
    /// having stronger requirements than the trait.
    Skip,
}

fn compare_asyncness<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    if tcx.asyncness(trait_m.def_id) == hir::IsAsync::Async {
        match tcx.fn_sig(impl_m.def_id).skip_binder().skip_binder().output().kind() {
            ty::Alias(ty::Opaque, ..) => {
                // allow both `async fn foo()` and `fn foo() -> impl Future`
            }
            ty::Error(_) => {
                // We don't know if it's ok, but at least it's already an error.
            }
            _ => {
                return Err(tcx
                    .sess
                    .create_err(crate::errors::AsyncTraitImplShouldBeAsync {
                        span: tcx.def_span(impl_m.def_id),
                        method_name: trait_m.name,
                        trait_item_span: tcx.hir().span_if_local(trait_m.def_id),
                    })
                    .emit_unless(delay));
            }
        };
    }

    Ok(())
}

/// Given a method def-id in an impl, compare the method signature of the impl
/// against the trait that it's implementing. In doing so, infer the hidden types
/// that this method's signature provides to satisfy each return-position `impl Trait`
/// in the trait signature.
///
/// The method is also responsible for making sure that the hidden types for each
/// RPITIT actually satisfy the bounds of the `impl Trait`, i.e. that if we infer
/// `impl Trait = Foo`, that `Foo: Trait` holds.
///
/// For example, given the sample code:
///
/// ```
/// #![feature(return_position_impl_trait_in_trait)]
///
/// use std::ops::Deref;
///
/// trait Foo {
///     fn bar() -> impl Deref<Target = impl Sized>;
///              // ^- RPITIT #1        ^- RPITIT #2
/// }
///
/// impl Foo for () {
///     fn bar() -> Box<String> { Box::new(String::new()) }
/// }
/// ```
///
/// The hidden types for the RPITITs in `bar` would be inferred to:
///     * `impl Deref` (RPITIT #1) = `Box<String>`
///     * `impl Sized` (RPITIT #2) = `String`
///
/// The relationship between these two types is straightforward in this case, but
/// may be more tenuously connected via other `impl`s and normalization rules for
/// cases of more complicated nested RPITITs.
#[instrument(skip(tcx), level = "debug", ret)]
pub(super) fn collect_return_position_impl_trait_in_trait_tys<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m_def_id: LocalDefId,
) -> Result<&'tcx FxHashMap<DefId, ty::EarlyBinder<Ty<'tcx>>>, ErrorGuaranteed> {
    let impl_m = tcx.opt_associated_item(impl_m_def_id.to_def_id()).unwrap();
    let trait_m = tcx.opt_associated_item(impl_m.trait_item_def_id.unwrap()).unwrap();
    let impl_trait_ref =
        tcx.impl_trait_ref(impl_m.impl_container(tcx).unwrap()).unwrap().instantiate_identity();
    let param_env = tcx.param_env(impl_m_def_id);

    // First, check a few of the same things as `compare_impl_method`,
    // just so we don't ICE during substitution later.
    check_method_is_structurally_compatible(tcx, impl_m, trait_m, impl_trait_ref, true)?;

    let trait_to_impl_args = impl_trait_ref.args;

    let impl_m_hir_id = tcx.hir().local_def_id_to_hir_id(impl_m_def_id);
    let return_span = tcx.hir().fn_decl_by_hir_id(impl_m_hir_id).unwrap().output.span();
    let cause = ObligationCause::new(
        return_span,
        impl_m_def_id,
        ObligationCauseCode::CompareImplItemObligation {
            impl_item_def_id: impl_m_def_id,
            trait_item_def_id: trait_m.def_id,
            kind: impl_m.kind,
        },
    );

    // Create mapping from impl to placeholder.
    let impl_to_placeholder_args = GenericArgs::identity_for_item(tcx, impl_m.def_id);

    // Create mapping from trait to placeholder.
    let trait_to_placeholder_args =
        impl_to_placeholder_args.rebase_onto(tcx, impl_m.container_id(tcx), trait_to_impl_args);

    let infcx = &tcx.infer_ctxt().build();
    let ocx = ObligationCtxt::new(infcx);

    // Normalize the impl signature with fresh variables for lifetime inference.
    let norm_cause = ObligationCause::misc(return_span, impl_m_def_id);
    let impl_sig = ocx.normalize(
        &norm_cause,
        param_env,
        tcx.liberate_late_bound_regions(
            impl_m.def_id,
            tcx.fn_sig(impl_m.def_id).instantiate_identity(),
        ),
    );
    impl_sig.error_reported()?;
    let impl_return_ty = impl_sig.output();

    // Normalize the trait signature with liberated bound vars, passing it through
    // the ImplTraitInTraitCollector, which gathers all of the RPITITs and replaces
    // them with inference variables.
    // We will use these inference variables to collect the hidden types of RPITITs.
    let mut collector = ImplTraitInTraitCollector::new(&ocx, return_span, param_env, impl_m_def_id);
    let unnormalized_trait_sig = infcx
        .instantiate_binder_with_fresh_vars(
            return_span,
            infer::HigherRankedType,
            tcx.fn_sig(trait_m.def_id).instantiate(tcx, trait_to_placeholder_args),
        )
        .fold_with(&mut collector);

    if !unnormalized_trait_sig.output().references_error() {
        debug_assert_ne!(
            collector.types.len(),
            0,
            "expect >1 RPITITs in call to `collect_return_position_impl_trait_in_trait_tys`"
        );
    }

    let trait_sig = ocx.normalize(&norm_cause, param_env, unnormalized_trait_sig);
    trait_sig.error_reported()?;
    let trait_return_ty = trait_sig.output();

    let wf_tys = FxIndexSet::from_iter(
        unnormalized_trait_sig.inputs_and_output.iter().chain(trait_sig.inputs_and_output.iter()),
    );

    match ocx.eq(&cause, param_env, trait_return_ty, impl_return_ty) {
        Ok(()) => {}
        Err(terr) => {
            let mut diag = struct_span_err!(
                tcx.sess,
                cause.span(),
                E0053,
                "method `{}` has an incompatible return type for trait",
                trait_m.name
            );
            let hir = tcx.hir();
            infcx.err_ctxt().note_type_err(
                &mut diag,
                &cause,
                hir.get_if_local(impl_m.def_id)
                    .and_then(|node| node.fn_decl())
                    .map(|decl| (decl.output.span(), Cow::from("return type in trait"))),
                Some(infer::ValuePairs::Terms(ExpectedFound {
                    expected: trait_return_ty.into(),
                    found: impl_return_ty.into(),
                })),
                terr,
                false,
                false,
            );
            return Err(diag.emit());
        }
    }

    debug!(?trait_sig, ?impl_sig, "equating function signatures");

    // Unify the whole function signature. We need to do this to fully infer
    // the lifetimes of the return type, but do this after unifying just the
    // return types, since we want to avoid duplicating errors from
    // `compare_method_predicate_entailment`.
    match ocx.eq(&cause, param_env, trait_sig, impl_sig) {
        Ok(()) => {}
        Err(terr) => {
            // This function gets called during `compare_method_predicate_entailment` when normalizing a
            // signature that contains RPITIT. When the method signatures don't match, we have to
            // emit an error now because `compare_method_predicate_entailment` will not report the error
            // when normalization fails.
            let emitted = report_trait_method_mismatch(
                infcx,
                cause,
                terr,
                (trait_m, trait_sig),
                (impl_m, impl_sig),
                impl_trait_ref,
            );
            return Err(emitted);
        }
    }

    // Check that all obligations are satisfied by the implementation's
    // RPITs.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        let reported = infcx.err_ctxt().report_fulfillment_errors(&errors);
        return Err(reported);
    }

    let collected_types = collector.types;

    // Finally, resolve all regions. This catches wily misuses of
    // lifetime parameters.
    let outlives_env = OutlivesEnvironment::with_bounds(
        param_env,
        infcx.implied_bounds_tys(param_env, impl_m_def_id, wf_tys),
    );
    ocx.resolve_regions_and_report_errors(impl_m_def_id, &outlives_env)?;

    let mut collected_tys = FxHashMap::default();
    for (def_id, (ty, args)) in collected_types {
        match infcx.fully_resolve((ty, args)) {
            Ok((ty, args)) => {
                // `ty` contains free regions that we created earlier while liberating the
                // trait fn signature. However, projection normalization expects `ty` to
                // contains `def_id`'s early-bound regions.
                let id_args = GenericArgs::identity_for_item(tcx, def_id);
                debug!(?id_args, ?args);
                let map: FxHashMap<_, _> = std::iter::zip(args, id_args)
                    .skip(tcx.generics_of(trait_m.def_id).count())
                    .filter_map(|(a, b)| Some((a.as_region()?, b.as_region()?)))
                    .collect();
                debug!(?map);

                // NOTE(compiler-errors): RPITITs, like all other RPITs, have early-bound
                // region args that are synthesized during AST lowering. These are args
                // that are appended to the parent args (trait and trait method). However,
                // we're trying to infer the unsubstituted type value of the RPITIT inside
                // the *impl*, so we can later use the impl's method args to normalize
                // an RPITIT to a concrete type (`confirm_impl_trait_in_trait_candidate`).
                //
                // Due to the design of RPITITs, during AST lowering, we have no idea that
                // an impl method corresponds to a trait method with RPITITs in it. Therefore,
                // we don't have a list of early-bound region args for the RPITIT in the impl.
                // Since early region parameters are index-based, we can't just rebase these
                // (trait method) early-bound region args onto the impl, and there's no
                // guarantee that the indices from the trait args and impl args line up.
                // So to fix this, we subtract the number of trait args and add the number of
                // impl args to *renumber* these early-bound regions to their corresponding
                // indices in the impl's substitutions list.
                //
                // Also, we only need to account for a difference in trait and impl args,
                // since we previously enforce that the trait method and impl method have the
                // same generics.
                let num_trait_args = trait_to_impl_args.len();
                let num_impl_args = tcx.generics_of(impl_m.container_id(tcx)).params.len();
                let ty = match ty.try_fold_with(&mut RemapHiddenTyRegions {
                    tcx,
                    map,
                    num_trait_args,
                    num_impl_args,
                    def_id,
                    impl_def_id: impl_m.container_id(tcx),
                    ty,
                    return_span,
                }) {
                    Ok(ty) => ty,
                    Err(guar) => Ty::new_error(tcx, guar),
                };
                collected_tys.insert(def_id, ty::EarlyBinder::bind(ty));
            }
            Err(err) => {
                let reported = tcx.sess.delay_span_bug(
                    return_span,
                    format!("could not fully resolve: {ty} => {err:?}"),
                );
                collected_tys.insert(def_id, ty::EarlyBinder::bind(Ty::new_error(tcx, reported)));
            }
        }
    }

    Ok(&*tcx.arena.alloc(collected_tys))
}

struct ImplTraitInTraitCollector<'a, 'tcx> {
    ocx: &'a ObligationCtxt<'a, 'tcx>,
    types: FxHashMap<DefId, (Ty<'tcx>, ty::GenericArgsRef<'tcx>)>,
    span: Span,
    param_env: ty::ParamEnv<'tcx>,
    body_id: LocalDefId,
}

impl<'a, 'tcx> ImplTraitInTraitCollector<'a, 'tcx> {
    fn new(
        ocx: &'a ObligationCtxt<'a, 'tcx>,
        span: Span,
        param_env: ty::ParamEnv<'tcx>,
        body_id: LocalDefId,
    ) -> Self {
        ImplTraitInTraitCollector { ocx, types: FxHashMap::default(), span, param_env, body_id }
    }
}

impl<'tcx> TypeFolder<TyCtxt<'tcx>> for ImplTraitInTraitCollector<'_, 'tcx> {
    fn interner(&self) -> TyCtxt<'tcx> {
        self.ocx.infcx.tcx
    }

    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        if let ty::Alias(ty::Projection, proj) = ty.kind()
            && self.interner().is_impl_trait_in_trait(proj.def_id)
        {
            if let Some((ty, _)) = self.types.get(&proj.def_id) {
                return *ty;
            }
            //FIXME(RPITIT): Deny nested RPITIT in args too
            if proj.args.has_escaping_bound_vars() {
                bug!("FIXME(RPITIT): error here");
            }
            // Replace with infer var
            let infer_ty = self.ocx.infcx.next_ty_var(TypeVariableOrigin {
                span: self.span,
                kind: TypeVariableOriginKind::MiscVariable,
            });
            self.types.insert(proj.def_id, (infer_ty, proj.args));
            // Recurse into bounds
            for (pred, pred_span) in self.interner().explicit_item_bounds(proj.def_id).iter_instantiated_copied(self.interner(), proj.args) {
                let pred = pred.fold_with(self);
                let pred = self.ocx.normalize(
                    &ObligationCause::misc(self.span, self.body_id),
                    self.param_env,
                    pred,
                );

                self.ocx.register_obligation(traits::Obligation::new(
                    self.interner(),
                    ObligationCause::new(
                        self.span,
                        self.body_id,
                        ObligationCauseCode::BindingObligation(proj.def_id, pred_span),
                    ),
                    self.param_env,
                    pred,
                ));
            }
            infer_ty
        } else {
            ty.super_fold_with(self)
        }
    }
}

struct RemapHiddenTyRegions<'tcx> {
    tcx: TyCtxt<'tcx>,
    map: FxHashMap<ty::Region<'tcx>, ty::Region<'tcx>>,
    num_trait_args: usize,
    num_impl_args: usize,
    def_id: DefId,
    impl_def_id: DefId,
    ty: Ty<'tcx>,
    return_span: Span,
}

impl<'tcx> ty::FallibleTypeFolder<TyCtxt<'tcx>> for RemapHiddenTyRegions<'tcx> {
    type Error = ErrorGuaranteed;

    fn interner(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn try_fold_ty(&mut self, t: Ty<'tcx>) -> Result<Ty<'tcx>, Self::Error> {
        if let ty::Alias(ty::Opaque, ty::AliasTy { args, def_id, .. }) = *t.kind() {
            let mut mapped_args = Vec::with_capacity(args.len());
            for (arg, v) in std::iter::zip(args, self.tcx.variances_of(def_id)) {
                mapped_args.push(match (arg.unpack(), v) {
                    // Skip uncaptured opaque args
                    (ty::GenericArgKind::Lifetime(_), ty::Bivariant) => arg,
                    _ => arg.try_fold_with(self)?,
                });
            }
            Ok(Ty::new_opaque(self.tcx, def_id, self.tcx.mk_args(&mapped_args)))
        } else {
            t.try_super_fold_with(self)
        }
    }

    fn try_fold_region(
        &mut self,
        region: ty::Region<'tcx>,
    ) -> Result<ty::Region<'tcx>, Self::Error> {
        match region.kind() {
            // Remap all free regions, which correspond to late-bound regions in the function.
            ty::ReFree(_) => {}
            // Remap early-bound regions as long as they don't come from the `impl` itself,
            // in which case we don't really need to renumber them.
            ty::ReEarlyBound(ebr) if self.tcx.parent(ebr.def_id) != self.impl_def_id => {}
            _ => return Ok(region),
        }

        let e = if let Some(region) = self.map.get(&region) {
            if let ty::ReEarlyBound(e) = region.kind() { e } else { bug!() }
        } else {
            let guar = match region.kind() {
                ty::ReEarlyBound(ty::EarlyBoundRegion { def_id, .. })
                | ty::ReFree(ty::FreeRegion {
                    bound_region: ty::BoundRegionKind::BrNamed(def_id, _),
                    ..
                }) => {
                    let return_span = if let ty::Alias(ty::Opaque, opaque_ty) = self.ty.kind() {
                        self.tcx.def_span(opaque_ty.def_id)
                    } else {
                        self.return_span
                    };
                    self.tcx
                        .sess
                        .struct_span_err(
                            return_span,
                            "return type captures more lifetimes than trait definition",
                        )
                        .span_label(self.tcx.def_span(def_id), "this lifetime was captured")
                        .span_note(
                            self.tcx.def_span(self.def_id),
                            "hidden type must only reference lifetimes captured by this impl trait",
                        )
                        .note(format!("hidden type inferred to be `{}`", self.ty))
                        .emit()
                }
                _ => self.tcx.sess.delay_span_bug(DUMMY_SP, "should've been able to remap region"),
            };
            return Err(guar);
        };

        Ok(ty::Region::new_early_bound(
            self.tcx,
            ty::EarlyBoundRegion {
                def_id: e.def_id,
                name: e.name,
                index: (e.index as usize - self.num_trait_args + self.num_impl_args) as u32,
            },
        ))
    }
}

fn report_trait_method_mismatch<'tcx>(
    infcx: &InferCtxt<'tcx>,
    mut cause: ObligationCause<'tcx>,
    terr: TypeError<'tcx>,
    (trait_m, trait_sig): (ty::AssocItem, ty::FnSig<'tcx>),
    (impl_m, impl_sig): (ty::AssocItem, ty::FnSig<'tcx>),
    impl_trait_ref: ty::TraitRef<'tcx>,
) -> ErrorGuaranteed {
    let tcx = infcx.tcx;
    let (impl_err_span, trait_err_span) =
        extract_spans_for_error_reporting(&infcx, terr, &cause, impl_m, trait_m);

    let mut diag = struct_span_err!(
        tcx.sess,
        impl_err_span,
        E0053,
        "method `{}` has an incompatible type for trait",
        trait_m.name
    );
    match &terr {
        TypeError::ArgumentMutability(0) | TypeError::ArgumentSorts(_, 0)
            if trait_m.fn_has_self_parameter =>
        {
            let ty = trait_sig.inputs()[0];
            let sugg = match ExplicitSelf::determine(ty, |ty| ty == impl_trait_ref.self_ty()) {
                ExplicitSelf::ByValue => "self".to_owned(),
                ExplicitSelf::ByReference(_, hir::Mutability::Not) => "&self".to_owned(),
                ExplicitSelf::ByReference(_, hir::Mutability::Mut) => "&mut self".to_owned(),
                _ => format!("self: {ty}"),
            };

            // When the `impl` receiver is an arbitrary self type, like `self: Box<Self>`, the
            // span points only at the type `Box<Self`>, but we want to cover the whole
            // argument pattern and type.
            let (sig, body) = tcx.hir().expect_impl_item(impl_m.def_id.expect_local()).expect_fn();
            let span = tcx
                .hir()
                .body_param_names(body)
                .zip(sig.decl.inputs.iter())
                .map(|(param, ty)| param.span.to(ty.span))
                .next()
                .unwrap_or(impl_err_span);

            diag.span_suggestion(
                span,
                "change the self-receiver type to match the trait",
                sugg,
                Applicability::MachineApplicable,
            );
        }
        TypeError::ArgumentMutability(i) | TypeError::ArgumentSorts(_, i) => {
            if trait_sig.inputs().len() == *i {
                // Suggestion to change output type. We do not suggest in `async` functions
                // to avoid complex logic or incorrect output.
                if let ImplItemKind::Fn(sig, _) = &tcx.hir().expect_impl_item(impl_m.def_id.expect_local()).kind
                    && !sig.header.asyncness.is_async()
                {
                    let msg = "change the output type to match the trait";
                    let ap = Applicability::MachineApplicable;
                    match sig.decl.output {
                        hir::FnRetTy::DefaultReturn(sp) => {
                            let sugg = format!("-> {} ", trait_sig.output());
                            diag.span_suggestion_verbose(sp, msg, sugg, ap);
                        }
                        hir::FnRetTy::Return(hir_ty) => {
                            let sugg = trait_sig.output();
                            diag.span_suggestion(hir_ty.span, msg, sugg, ap);
                        }
                    };
                };
            } else if let Some(trait_ty) = trait_sig.inputs().get(*i) {
                diag.span_suggestion(
                    impl_err_span,
                    "change the parameter type to match the trait",
                    trait_ty,
                    Applicability::MachineApplicable,
                );
            }
        }
        _ => {}
    }

    cause.span = impl_err_span;
    infcx.err_ctxt().note_type_err(
        &mut diag,
        &cause,
        trait_err_span.map(|sp| (sp, Cow::from("type in trait"))),
        Some(infer::ValuePairs::Sigs(ExpectedFound { expected: trait_sig, found: impl_sig })),
        terr,
        false,
        false,
    );

    return diag.emit();
}

fn check_region_bounds_on_impl_item<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    let impl_generics = tcx.generics_of(impl_m.def_id);
    let impl_params = impl_generics.own_counts().lifetimes;

    let trait_generics = tcx.generics_of(trait_m.def_id);
    let trait_params = trait_generics.own_counts().lifetimes;

    debug!(
        "check_region_bounds_on_impl_item: \
            trait_generics={:?} \
            impl_generics={:?}",
        trait_generics, impl_generics
    );

    // Must have same number of early-bound lifetime parameters.
    // Unfortunately, if the user screws up the bounds, then this
    // will change classification between early and late. E.g.,
    // if in trait we have `<'a,'b:'a>`, and in impl we just have
    // `<'a,'b>`, then we have 2 early-bound lifetime parameters
    // in trait but 0 in the impl. But if we report "expected 2
    // but found 0" it's confusing, because it looks like there
    // are zero. Since I don't quite know how to phrase things at
    // the moment, give a kind of vague error message.
    if trait_params != impl_params {
        let span = tcx
            .hir()
            .get_generics(impl_m.def_id.expect_local())
            .expect("expected impl item to have generics or else we can't compare them")
            .span;

        let mut generics_span = None;
        let mut bounds_span = vec![];
        let mut where_span = None;
        if let Some(trait_node) = tcx.hir().get_if_local(trait_m.def_id)
            && let Some(trait_generics) = trait_node.generics()
        {
            generics_span = Some(trait_generics.span);
            // FIXME: we could potentially look at the impl's bounds to not point at bounds that
            // *are* present in the impl.
            for p in trait_generics.predicates {
                if let hir::WherePredicate::BoundPredicate(pred) = p {
                    for b in pred.bounds {
                        if let hir::GenericBound::Outlives(lt) = b {
                            bounds_span.push(lt.ident.span);
                        }
                    }
                }
            }
            if let Some(impl_node) = tcx.hir().get_if_local(impl_m.def_id)
                && let Some(impl_generics) = impl_node.generics()
            {
                let mut impl_bounds = 0;
                for p in impl_generics.predicates {
                    if let hir::WherePredicate::BoundPredicate(pred) = p {
                        for b in pred.bounds {
                            if let hir::GenericBound::Outlives(_) = b {
                                impl_bounds += 1;
                            }
                        }
                    }
                }
                if impl_bounds == bounds_span.len() {
                    bounds_span = vec![];
                } else if impl_generics.has_where_clause_predicates {
                    where_span = Some(impl_generics.where_clause_span);
                }
            }
        }
        let reported = tcx
            .sess
            .create_err(LifetimesOrBoundsMismatchOnTrait {
                span,
                item_kind: assoc_item_kind_str(&impl_m),
                ident: impl_m.ident(tcx),
                generics_span,
                bounds_span,
                where_span,
            })
            .emit_unless(delay);
        return Err(reported);
    }

    Ok(())
}

#[instrument(level = "debug", skip(infcx))]
fn extract_spans_for_error_reporting<'tcx>(
    infcx: &infer::InferCtxt<'tcx>,
    terr: TypeError<'_>,
    cause: &ObligationCause<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
) -> (Span, Option<Span>) {
    let tcx = infcx.tcx;
    let mut impl_args = {
        let (sig, _) = tcx.hir().expect_impl_item(impl_m.def_id.expect_local()).expect_fn();
        sig.decl.inputs.iter().map(|t| t.span).chain(iter::once(sig.decl.output.span()))
    };

    let trait_args = trait_m.def_id.as_local().map(|def_id| {
        let (sig, _) = tcx.hir().expect_trait_item(def_id).expect_fn();
        sig.decl.inputs.iter().map(|t| t.span).chain(iter::once(sig.decl.output.span()))
    });

    match terr {
        TypeError::ArgumentMutability(i) | TypeError::ArgumentSorts(ExpectedFound { .. }, i) => {
            (impl_args.nth(i).unwrap(), trait_args.and_then(|mut args| args.nth(i)))
        }
        _ => (cause.span(), tcx.hir().span_if_local(trait_m.def_id)),
    }
}

fn compare_self_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    // Try to give more informative error messages about self typing
    // mismatches. Note that any mismatch will also be detected
    // below, where we construct a canonical function type that
    // includes the self parameter as a normal parameter. It's just
    // that the error messages you get out of this code are a bit more
    // inscrutable, particularly for cases where one method has no
    // self.

    let self_string = |method: ty::AssocItem| {
        let untransformed_self_ty = match method.container {
            ty::ImplContainer => impl_trait_ref.self_ty(),
            ty::TraitContainer => tcx.types.self_param,
        };
        let self_arg_ty = tcx.fn_sig(method.def_id).instantiate_identity().input(0);
        let param_env = ty::ParamEnv::reveal_all();

        let infcx = tcx.infer_ctxt().build();
        let self_arg_ty = tcx.liberate_late_bound_regions(method.def_id, self_arg_ty);
        let can_eq_self = |ty| infcx.can_eq(param_env, untransformed_self_ty, ty);
        match ExplicitSelf::determine(self_arg_ty, can_eq_self) {
            ExplicitSelf::ByValue => "self".to_owned(),
            ExplicitSelf::ByReference(_, hir::Mutability::Not) => "&self".to_owned(),
            ExplicitSelf::ByReference(_, hir::Mutability::Mut) => "&mut self".to_owned(),
            _ => format!("self: {self_arg_ty}"),
        }
    };

    match (trait_m.fn_has_self_parameter, impl_m.fn_has_self_parameter) {
        (false, false) | (true, true) => {}

        (false, true) => {
            let self_descr = self_string(impl_m);
            let impl_m_span = tcx.def_span(impl_m.def_id);
            let mut err = struct_span_err!(
                tcx.sess,
                impl_m_span,
                E0185,
                "method `{}` has a `{}` declaration in the impl, but not in the trait",
                trait_m.name,
                self_descr
            );
            err.span_label(impl_m_span, format!("`{self_descr}` used in impl"));
            if let Some(span) = tcx.hir().span_if_local(trait_m.def_id) {
                err.span_label(span, format!("trait method declared without `{self_descr}`"));
            } else {
                err.note_trait_signature(trait_m.name, trait_m.signature(tcx));
            }
            return Err(err.emit_unless(delay));
        }

        (true, false) => {
            let self_descr = self_string(trait_m);
            let impl_m_span = tcx.def_span(impl_m.def_id);
            let mut err = struct_span_err!(
                tcx.sess,
                impl_m_span,
                E0186,
                "method `{}` has a `{}` declaration in the trait, but not in the impl",
                trait_m.name,
                self_descr
            );
            err.span_label(impl_m_span, format!("expected `{self_descr}` in impl"));
            if let Some(span) = tcx.hir().span_if_local(trait_m.def_id) {
                err.span_label(span, format!("`{self_descr}` used in trait"));
            } else {
                err.note_trait_signature(trait_m.name, trait_m.signature(tcx));
            }

            return Err(err.emit_unless(delay));
        }
    }

    Ok(())
}

/// Checks that the number of generics on a given assoc item in a trait impl is the same
/// as the number of generics on the respective assoc item in the trait definition.
///
/// For example this code emits the errors in the following code:
/// ```rust,compile_fail
/// trait Trait {
///     fn foo();
///     type Assoc<T>;
/// }
///
/// impl Trait for () {
///     fn foo<T>() {}
///     //~^ error
///     type Assoc = u32;
///     //~^ error
/// }
/// ```
///
/// Notably this does not error on `foo<T>` implemented as `foo<const N: u8>` or
/// `foo<const N: u8>` implemented as `foo<const N: u32>`. This is handled in
/// [`compare_generic_param_kinds`]. This function also does not handle lifetime parameters
fn compare_number_of_generics<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_: ty::AssocItem,
    trait_: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    let trait_own_counts = tcx.generics_of(trait_.def_id).own_counts();
    let impl_own_counts = tcx.generics_of(impl_.def_id).own_counts();

    // This avoids us erroring on `foo<T>` implemented as `foo<const N: u8>` as this is implemented
    // in `compare_generic_param_kinds` which will give a nicer error message than something like:
    // "expected 1 type parameter, found 0 type parameters"
    if (trait_own_counts.types + trait_own_counts.consts)
        == (impl_own_counts.types + impl_own_counts.consts)
    {
        return Ok(());
    }

    // We never need to emit a separate error for RPITITs, since if an RPITIT
    // has mismatched type or const generic arguments, then the method that it's
    // inheriting the generics from will also have mismatched arguments, and
    // we'll report an error for that instead. Delay a bug for safety, though.
    if trait_.is_impl_trait_in_trait() {
        return Err(tcx.sess.delay_span_bug(
            rustc_span::DUMMY_SP,
            "errors comparing numbers of generics of trait/impl functions were not emitted",
        ));
    }

    let matchings = [
        ("type", trait_own_counts.types, impl_own_counts.types),
        ("const", trait_own_counts.consts, impl_own_counts.consts),
    ];

    let item_kind = assoc_item_kind_str(&impl_);

    let mut err_occurred = None;
    for (kind, trait_count, impl_count) in matchings {
        if impl_count != trait_count {
            let arg_spans = |kind: ty::AssocKind, generics: &hir::Generics<'_>| {
                let mut spans = generics
                    .params
                    .iter()
                    .filter(|p| match p.kind {
                        hir::GenericParamKind::Lifetime {
                            kind: hir::LifetimeParamKind::Elided,
                        } => {
                            // A fn can have an arbitrary number of extra elided lifetimes for the
                            // same signature.
                            !matches!(kind, ty::AssocKind::Fn)
                        }
                        _ => true,
                    })
                    .map(|p| p.span)
                    .collect::<Vec<Span>>();
                if spans.is_empty() {
                    spans = vec![generics.span]
                }
                spans
            };
            let (trait_spans, impl_trait_spans) = if let Some(def_id) = trait_.def_id.as_local() {
                let trait_item = tcx.hir().expect_trait_item(def_id);
                let arg_spans: Vec<Span> = arg_spans(trait_.kind, trait_item.generics);
                let impl_trait_spans: Vec<Span> = trait_item
                    .generics
                    .params
                    .iter()
                    .filter_map(|p| match p.kind {
                        GenericParamKind::Type { synthetic: true, .. } => Some(p.span),
                        _ => None,
                    })
                    .collect();
                (Some(arg_spans), impl_trait_spans)
            } else {
                let trait_span = tcx.hir().span_if_local(trait_.def_id);
                (trait_span.map(|s| vec![s]), vec![])
            };

            let impl_item = tcx.hir().expect_impl_item(impl_.def_id.expect_local());
            let impl_item_impl_trait_spans: Vec<Span> = impl_item
                .generics
                .params
                .iter()
                .filter_map(|p| match p.kind {
                    GenericParamKind::Type { synthetic: true, .. } => Some(p.span),
                    _ => None,
                })
                .collect();
            let spans = arg_spans(impl_.kind, impl_item.generics);
            let span = spans.first().copied();

            let mut err = tcx.sess.struct_span_err_with_code(
                spans,
                format!(
                    "{} `{}` has {} {kind} parameter{} but its trait \
                     declaration has {} {kind} parameter{}",
                    item_kind,
                    trait_.name,
                    impl_count,
                    pluralize!(impl_count),
                    trait_count,
                    pluralize!(trait_count),
                    kind = kind,
                ),
                DiagnosticId::Error("E0049".into()),
            );

            let mut suffix = None;

            if let Some(spans) = trait_spans {
                let mut spans = spans.iter();
                if let Some(span) = spans.next() {
                    err.span_label(
                        *span,
                        format!(
                            "expected {} {} parameter{}",
                            trait_count,
                            kind,
                            pluralize!(trait_count),
                        ),
                    );
                }
                for span in spans {
                    err.span_label(*span, "");
                }
            } else {
                suffix = Some(format!(", expected {trait_count}"));
            }

            if let Some(span) = span {
                err.span_label(
                    span,
                    format!(
                        "found {} {} parameter{}{}",
                        impl_count,
                        kind,
                        pluralize!(impl_count),
                        suffix.unwrap_or_default(),
                    ),
                );
            }

            for span in impl_trait_spans.iter().chain(impl_item_impl_trait_spans.iter()) {
                err.span_label(*span, "`impl Trait` introduces an implicit type parameter");
            }

            let reported = err.emit_unless(delay);
            err_occurred = Some(reported);
        }
    }

    if let Some(reported) = err_occurred { Err(reported) } else { Ok(()) }
}

fn compare_number_of_method_arguments<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    let impl_m_fty = tcx.fn_sig(impl_m.def_id);
    let trait_m_fty = tcx.fn_sig(trait_m.def_id);
    let trait_number_args = trait_m_fty.skip_binder().inputs().skip_binder().len();
    let impl_number_args = impl_m_fty.skip_binder().inputs().skip_binder().len();

    if trait_number_args != impl_number_args {
        let trait_span = trait_m
            .def_id
            .as_local()
            .and_then(|def_id| {
                let (trait_m_sig, _) = &tcx.hir().expect_trait_item(def_id).expect_fn();
                let pos = trait_number_args.saturating_sub(1);
                trait_m_sig.decl.inputs.get(pos).map(|arg| {
                    if pos == 0 {
                        arg.span
                    } else {
                        arg.span.with_lo(trait_m_sig.decl.inputs[0].span.lo())
                    }
                })
            })
            .or_else(|| tcx.hir().span_if_local(trait_m.def_id));

        let (impl_m_sig, _) = &tcx.hir().expect_impl_item(impl_m.def_id.expect_local()).expect_fn();
        let pos = impl_number_args.saturating_sub(1);
        let impl_span = impl_m_sig
            .decl
            .inputs
            .get(pos)
            .map(|arg| {
                if pos == 0 {
                    arg.span
                } else {
                    arg.span.with_lo(impl_m_sig.decl.inputs[0].span.lo())
                }
            })
            .unwrap_or_else(|| tcx.def_span(impl_m.def_id));

        let mut err = struct_span_err!(
            tcx.sess,
            impl_span,
            E0050,
            "method `{}` has {} but the declaration in trait `{}` has {}",
            trait_m.name,
            potentially_plural_count(impl_number_args, "parameter"),
            tcx.def_path_str(trait_m.def_id),
            trait_number_args
        );

        if let Some(trait_span) = trait_span {
            err.span_label(
                trait_span,
                format!(
                    "trait requires {}",
                    potentially_plural_count(trait_number_args, "parameter")
                ),
            );
        } else {
            err.note_trait_signature(trait_m.name, trait_m.signature(tcx));
        }

        err.span_label(
            impl_span,
            format!(
                "expected {}, found {}",
                potentially_plural_count(trait_number_args, "parameter"),
                impl_number_args
            ),
        );

        return Err(err.emit_unless(delay));
    }

    Ok(())
}

fn compare_synthetic_generics<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_m: ty::AssocItem,
    trait_m: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    // FIXME(chrisvittal) Clean up this function, list of FIXME items:
    //     1. Better messages for the span labels
    //     2. Explanation as to what is going on
    // If we get here, we already have the same number of generics, so the zip will
    // be okay.
    let mut error_found = None;
    let impl_m_generics = tcx.generics_of(impl_m.def_id);
    let trait_m_generics = tcx.generics_of(trait_m.def_id);
    let impl_m_type_params = impl_m_generics.params.iter().filter_map(|param| match param.kind {
        GenericParamDefKind::Type { synthetic, .. } => Some((param.def_id, synthetic)),
        GenericParamDefKind::Lifetime | GenericParamDefKind::Const { .. } => None,
    });
    let trait_m_type_params = trait_m_generics.params.iter().filter_map(|param| match param.kind {
        GenericParamDefKind::Type { synthetic, .. } => Some((param.def_id, synthetic)),
        GenericParamDefKind::Lifetime | GenericParamDefKind::Const { .. } => None,
    });
    for ((impl_def_id, impl_synthetic), (trait_def_id, trait_synthetic)) in
        iter::zip(impl_m_type_params, trait_m_type_params)
    {
        if impl_synthetic != trait_synthetic {
            let impl_def_id = impl_def_id.expect_local();
            let impl_span = tcx.def_span(impl_def_id);
            let trait_span = tcx.def_span(trait_def_id);
            let mut err = struct_span_err!(
                tcx.sess,
                impl_span,
                E0643,
                "method `{}` has incompatible signature for trait",
                trait_m.name
            );
            err.span_label(trait_span, "declaration in trait here");
            match (impl_synthetic, trait_synthetic) {
                // The case where the impl method uses `impl Trait` but the trait method uses
                // explicit generics
                (true, false) => {
                    err.span_label(impl_span, "expected generic parameter, found `impl Trait`");
                    let _: Option<_> = try {
                        // try taking the name from the trait impl
                        // FIXME: this is obviously suboptimal since the name can already be used
                        // as another generic argument
                        let new_name = tcx.opt_item_name(trait_def_id)?;
                        let trait_m = trait_m.def_id.as_local()?;
                        let trait_m = tcx.hir().expect_trait_item(trait_m);

                        let impl_m = impl_m.def_id.as_local()?;
                        let impl_m = tcx.hir().expect_impl_item(impl_m);

                        // in case there are no generics, take the spot between the function name
                        // and the opening paren of the argument list
                        let new_generics_span = tcx.def_ident_span(impl_def_id)?.shrink_to_hi();
                        // in case there are generics, just replace them
                        let generics_span =
                            impl_m.generics.span.substitute_dummy(new_generics_span);
                        // replace with the generics from the trait
                        let new_generics =
                            tcx.sess.source_map().span_to_snippet(trait_m.generics.span).ok()?;

                        err.multipart_suggestion(
                            "try changing the `impl Trait` argument to a generic parameter",
                            vec![
                                // replace `impl Trait` with `T`
                                (impl_span, new_name.to_string()),
                                // replace impl method generics with trait method generics
                                // This isn't quite right, as users might have changed the names
                                // of the generics, but it works for the common case
                                (generics_span, new_generics),
                            ],
                            Applicability::MaybeIncorrect,
                        );
                    };
                }
                // The case where the trait method uses `impl Trait`, but the impl method uses
                // explicit generics.
                (false, true) => {
                    err.span_label(impl_span, "expected `impl Trait`, found generic parameter");
                    let _: Option<_> = try {
                        let impl_m = impl_m.def_id.as_local()?;
                        let impl_m = tcx.hir().expect_impl_item(impl_m);
                        let (sig, _) = impl_m.expect_fn();
                        let input_tys = sig.decl.inputs;

                        struct Visitor(Option<Span>, hir::def_id::LocalDefId);
                        impl<'v> intravisit::Visitor<'v> for Visitor {
                            fn visit_ty(&mut self, ty: &'v hir::Ty<'v>) {
                                intravisit::walk_ty(self, ty);
                                if let hir::TyKind::Path(hir::QPath::Resolved(None, path)) = ty.kind
                                    && let Res::Def(DefKind::TyParam, def_id) = path.res
                                    && def_id == self.1.to_def_id()
                                {
                                    self.0 = Some(ty.span);
                                }
                            }
                        }

                        let mut visitor = Visitor(None, impl_def_id);
                        for ty in input_tys {
                            intravisit::Visitor::visit_ty(&mut visitor, ty);
                        }
                        let span = visitor.0?;

                        let bounds = impl_m.generics.bounds_for_param(impl_def_id).next()?.bounds;
                        let bounds = bounds.first()?.span().to(bounds.last()?.span());
                        let bounds = tcx.sess.source_map().span_to_snippet(bounds).ok()?;

                        err.multipart_suggestion(
                            "try removing the generic parameter and using `impl Trait` instead",
                            vec![
                                // delete generic parameters
                                (impl_m.generics.span, String::new()),
                                // replace param usage with `impl Trait`
                                (span, format!("impl {bounds}")),
                            ],
                            Applicability::MaybeIncorrect,
                        );
                    };
                }
                _ => unreachable!(),
            }
            error_found = Some(err.emit_unless(delay));
        }
    }
    if let Some(reported) = error_found { Err(reported) } else { Ok(()) }
}

/// Checks that all parameters in the generics of a given assoc item in a trait impl have
/// the same kind as the respective generic parameter in the trait def.
///
/// For example all 4 errors in the following code are emitted here:
/// ```rust,ignore (pseudo-Rust)
/// trait Foo {
///     fn foo<const N: u8>();
///     type bar<const N: u8>;
///     fn baz<const N: u32>();
///     type blah<T>;
/// }
///
/// impl Foo for () {
///     fn foo<const N: u64>() {}
///     //~^ error
///     type bar<const N: u64> {}
///     //~^ error
///     fn baz<T>() {}
///     //~^ error
///     type blah<const N: i64> = u32;
///     //~^ error
/// }
/// ```
///
/// This function does not handle lifetime parameters
fn compare_generic_param_kinds<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_item: ty::AssocItem,
    trait_item: ty::AssocItem,
    delay: bool,
) -> Result<(), ErrorGuaranteed> {
    assert_eq!(impl_item.kind, trait_item.kind);

    let ty_const_params_of = |def_id| {
        tcx.generics_of(def_id).params.iter().filter(|param| {
            matches!(
                param.kind,
                GenericParamDefKind::Const { .. } | GenericParamDefKind::Type { .. }
            )
        })
    };

    for (param_impl, param_trait) in
        iter::zip(ty_const_params_of(impl_item.def_id), ty_const_params_of(trait_item.def_id))
    {
        use GenericParamDefKind::*;
        if match (&param_impl.kind, &param_trait.kind) {
            (Const { .. }, Const { .. })
                if tcx.type_of(param_impl.def_id) != tcx.type_of(param_trait.def_id) =>
            {
                true
            }
            (Const { .. }, Type { .. }) | (Type { .. }, Const { .. }) => true,
            // this is exhaustive so that anyone adding new generic param kinds knows
            // to make sure this error is reported for them.
            (Const { .. }, Const { .. }) | (Type { .. }, Type { .. }) => false,
            (Lifetime { .. }, _) | (_, Lifetime { .. }) => unreachable!(),
        } {
            let param_impl_span = tcx.def_span(param_impl.def_id);
            let param_trait_span = tcx.def_span(param_trait.def_id);

            let mut err = struct_span_err!(
                tcx.sess,
                param_impl_span,
                E0053,
                "{} `{}` has an incompatible generic parameter for trait `{}`",
                assoc_item_kind_str(&impl_item),
                trait_item.name,
                &tcx.def_path_str(tcx.parent(trait_item.def_id))
            );

            let make_param_message = |prefix: &str, param: &ty::GenericParamDef| match param.kind {
                Const { .. } => {
                    format!(
                        "{} const parameter of type `{}`",
                        prefix,
                        tcx.type_of(param.def_id).instantiate_identity()
                    )
                }
                Type { .. } => format!("{prefix} type parameter"),
                Lifetime { .. } => unreachable!(),
            };

            let trait_header_span = tcx.def_ident_span(tcx.parent(trait_item.def_id)).unwrap();
            err.span_label(trait_header_span, "");
            err.span_label(param_trait_span, make_param_message("expected", param_trait));

            let impl_header_span = tcx.def_span(tcx.parent(impl_item.def_id));
            err.span_label(impl_header_span, "");
            err.span_label(param_impl_span, make_param_message("found", param_impl));

            let reported = err.emit_unless(delay);
            return Err(reported);
        }
    }

    Ok(())
}

/// Use `tcx.compare_impl_const` instead
pub(super) fn compare_impl_const_raw(
    tcx: TyCtxt<'_>,
    (impl_const_item_def, trait_const_item_def): (LocalDefId, DefId),
) -> Result<(), ErrorGuaranteed> {
    let impl_const_item = tcx.associated_item(impl_const_item_def);
    let trait_const_item = tcx.associated_item(trait_const_item_def);
    let impl_trait_ref =
        tcx.impl_trait_ref(impl_const_item.container_id(tcx)).unwrap().instantiate_identity();
    debug!("compare_const_impl(impl_trait_ref={:?})", impl_trait_ref);

    let impl_c_span = tcx.def_span(impl_const_item_def.to_def_id());

    let infcx = tcx.infer_ctxt().build();
    let param_env = tcx.param_env(impl_const_item_def.to_def_id());
    let ocx = ObligationCtxt::new(&infcx);

    // The below is for the most part highly similar to the procedure
    // for methods above. It is simpler in many respects, especially
    // because we shouldn't really have to deal with lifetimes or
    // predicates. In fact some of this should probably be put into
    // shared functions because of DRY violations...
    let trait_to_impl_args = impl_trait_ref.args;

    // Create a parameter environment that represents the implementation's
    // method.
    // Compute placeholder form of impl and trait const tys.
    let impl_ty = tcx.type_of(impl_const_item_def.to_def_id()).instantiate_identity();
    let trait_ty = tcx.type_of(trait_const_item_def).instantiate(tcx, trait_to_impl_args);
    let mut cause = ObligationCause::new(
        impl_c_span,
        impl_const_item_def,
        ObligationCauseCode::CompareImplItemObligation {
            impl_item_def_id: impl_const_item_def,
            trait_item_def_id: trait_const_item_def,
            kind: impl_const_item.kind,
        },
    );

    // There is no "body" here, so just pass dummy id.
    let impl_ty = ocx.normalize(&cause, param_env, impl_ty);

    debug!("compare_const_impl: impl_ty={:?}", impl_ty);

    let trait_ty = ocx.normalize(&cause, param_env, trait_ty);

    debug!("compare_const_impl: trait_ty={:?}", trait_ty);

    let err = ocx.sup(&cause, param_env, trait_ty, impl_ty);

    if let Err(terr) = err {
        debug!(
            "checking associated const for compatibility: impl ty {:?}, trait ty {:?}",
            impl_ty, trait_ty
        );

        // Locate the Span containing just the type of the offending impl
        let (ty, _) = tcx.hir().expect_impl_item(impl_const_item_def).expect_const();
        cause.span = ty.span;

        let mut diag = struct_span_err!(
            tcx.sess,
            cause.span,
            E0326,
            "implemented const `{}` has an incompatible type for trait",
            trait_const_item.name
        );

        let trait_c_span = trait_const_item_def.as_local().map(|trait_c_def_id| {
            // Add a label to the Span containing just the type of the const
            let (ty, _) = tcx.hir().expect_trait_item(trait_c_def_id).expect_const();
            ty.span
        });

        infcx.err_ctxt().note_type_err(
            &mut diag,
            &cause,
            trait_c_span.map(|span| (span, Cow::from("type in trait"))),
            Some(infer::ValuePairs::Terms(ExpectedFound {
                expected: trait_ty.into(),
                found: impl_ty.into(),
            })),
            terr,
            false,
            false,
        );
        return Err(diag.emit());
    };

    // Check that all obligations are satisfied by the implementation's
    // version.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        return Err(infcx.err_ctxt().report_fulfillment_errors(&errors));
    }

    let outlives_env = OutlivesEnvironment::new(param_env);
    ocx.resolve_regions_and_report_errors(impl_const_item_def, &outlives_env)
}

pub(super) fn compare_impl_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_ty: ty::AssocItem,
    trait_ty: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
) {
    debug!("compare_impl_type(impl_trait_ref={:?})", impl_trait_ref);

    let _: Result<(), ErrorGuaranteed> = try {
        compare_number_of_generics(tcx, impl_ty, trait_ty, false)?;
        compare_generic_param_kinds(tcx, impl_ty, trait_ty, false)?;
        compare_type_predicate_entailment(tcx, impl_ty, trait_ty, impl_trait_ref)?;
        check_type_bounds(tcx, trait_ty, impl_ty, impl_trait_ref)?;
    };
}

/// The equivalent of [compare_method_predicate_entailment], but for associated types
/// instead of associated functions.
fn compare_type_predicate_entailment<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_ty: ty::AssocItem,
    trait_ty: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
) -> Result<(), ErrorGuaranteed> {
    let impl_args = GenericArgs::identity_for_item(tcx, impl_ty.def_id);
    let trait_to_impl_args =
        impl_args.rebase_onto(tcx, impl_ty.container_id(tcx), impl_trait_ref.args);

    let impl_ty_predicates = tcx.predicates_of(impl_ty.def_id);
    let trait_ty_predicates = tcx.predicates_of(trait_ty.def_id);

    check_region_bounds_on_impl_item(tcx, impl_ty, trait_ty, false)?;

    let impl_ty_own_bounds = impl_ty_predicates.instantiate_own(tcx, impl_args);
    if impl_ty_own_bounds.len() == 0 {
        // Nothing to check.
        return Ok(());
    }

    // This `HirId` should be used for the `body_id` field on each
    // `ObligationCause` (and the `FnCtxt`). This is what
    // `regionck_item` expects.
    let impl_ty_def_id = impl_ty.def_id.expect_local();
    debug!("compare_type_predicate_entailment: trait_to_impl_args={:?}", trait_to_impl_args);

    // The predicates declared by the impl definition, the trait and the
    // associated type in the trait are assumed.
    let impl_predicates = tcx.predicates_of(impl_ty_predicates.parent.unwrap());
    let mut hybrid_preds = impl_predicates.instantiate_identity(tcx);
    hybrid_preds.predicates.extend(
        trait_ty_predicates
            .instantiate_own(tcx, trait_to_impl_args)
            .map(|(predicate, _)| predicate),
    );

    debug!("compare_type_predicate_entailment: bounds={:?}", hybrid_preds);

    let impl_ty_span = tcx.def_span(impl_ty_def_id);
    let normalize_cause = traits::ObligationCause::misc(impl_ty_span, impl_ty_def_id);
    let param_env = ty::ParamEnv::new(tcx.mk_clauses(&hybrid_preds.predicates), Reveal::UserFacing);
    let param_env = traits::normalize_param_env_or_error(tcx, param_env, normalize_cause);
    let infcx = tcx.infer_ctxt().build();
    let ocx = ObligationCtxt::new(&infcx);

    debug!("compare_type_predicate_entailment: caller_bounds={:?}", param_env.caller_bounds());

    for (predicate, span) in impl_ty_own_bounds {
        let cause = ObligationCause::misc(span, impl_ty_def_id);
        let predicate = ocx.normalize(&cause, param_env, predicate);

        let cause = ObligationCause::new(
            span,
            impl_ty_def_id,
            ObligationCauseCode::CompareImplItemObligation {
                impl_item_def_id: impl_ty.def_id.expect_local(),
                trait_item_def_id: trait_ty.def_id,
                kind: impl_ty.kind,
            },
        );
        ocx.register_obligation(traits::Obligation::new(tcx, cause, param_env, predicate));
    }

    // Check that all obligations are satisfied by the implementation's
    // version.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        let reported = infcx.err_ctxt().report_fulfillment_errors(&errors);
        return Err(reported);
    }

    // Finally, resolve all regions. This catches wily misuses of
    // lifetime parameters.
    let outlives_env = OutlivesEnvironment::new(param_env);
    ocx.resolve_regions_and_report_errors(impl_ty_def_id, &outlives_env)
}

/// Validate that `ProjectionCandidate`s created for this associated type will
/// be valid.
///
/// Usually given
///
/// trait X { type Y: Copy } impl X for T { type Y = S; }
///
/// We are able to normalize `<T as X>::U` to `S`, and so when we check the
/// impl is well-formed we have to prove `S: Copy`.
///
/// For default associated types the normalization is not possible (the value
/// from the impl could be overridden). We also can't normalize generic
/// associated types (yet) because they contain bound parameters.
#[instrument(level = "debug", skip(tcx))]
pub(super) fn check_type_bounds<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_ty: ty::AssocItem,
    impl_ty: ty::AssocItem,
    impl_trait_ref: ty::TraitRef<'tcx>,
) -> Result<(), ErrorGuaranteed> {
    let param_env = tcx.param_env(impl_ty.def_id);
    let container_id = impl_ty.container_id(tcx);
    // Given
    //
    // impl<A, B> Foo<u32> for (A, B) {
    //     type Bar<C> = Wrapper<A, B, C>
    // }
    //
    // - `impl_trait_ref` would be `<(A, B) as Foo<u32>>`
    // - `normalize_impl_ty_args` would be `[A, B, ^0.0]` (`^0.0` here is the bound var with db 0 and index 0)
    // - `normalize_impl_ty` would be `Wrapper<A, B, ^0.0>`
    // - `rebased_args` would be `[(A, B), u32, ^0.0]`, combining the args from
    //    the *trait* with the generic associated type parameters (as bound vars).
    //
    // A note regarding the use of bound vars here:
    // Imagine as an example
    // ```
    // trait Family {
    //     type Member<C: Eq>;
    // }
    //
    // impl Family for VecFamily {
    //     type Member<C: Eq> = i32;
    // }
    // ```
    // Here, we would generate
    // ```notrust
    // forall<C> { Normalize(<VecFamily as Family>::Member<C> => i32) }
    // ```
    // when we really would like to generate
    // ```notrust
    // forall<C> { Normalize(<VecFamily as Family>::Member<C> => i32) :- Implemented(C: Eq) }
    // ```
    // But, this is probably fine, because although the first clause can be used with types C that
    // do not implement Eq, for it to cause some kind of problem, there would have to be a
    // VecFamily::Member<X> for some type X where !(X: Eq), that appears in the value of type
    // Member<C: Eq> = .... That type would fail a well-formedness check that we ought to be doing
    // elsewhere, which would check that any <T as Family>::Member<X> meets the bounds declared in
    // the trait (notably, that X: Eq and T: Family).
    let mut bound_vars: smallvec::SmallVec<[ty::BoundVariableKind; 8]> =
        smallvec::SmallVec::with_capacity(tcx.generics_of(impl_ty.def_id).params.len());
    // Extend the impl's identity args with late-bound GAT vars
    let normalize_impl_ty_args = ty::GenericArgs::identity_for_item(tcx, container_id).extend_to(
        tcx,
        impl_ty.def_id,
        |param, _| match param.kind {
            GenericParamDefKind::Type { .. } => {
                let kind = ty::BoundTyKind::Param(param.def_id, param.name);
                let bound_var = ty::BoundVariableKind::Ty(kind);
                bound_vars.push(bound_var);
                Ty::new_bound(
                    tcx,
                    ty::INNERMOST,
                    ty::BoundTy { var: ty::BoundVar::from_usize(bound_vars.len() - 1), kind },
                )
                .into()
            }
            GenericParamDefKind::Lifetime => {
                let kind = ty::BoundRegionKind::BrNamed(param.def_id, param.name);
                let bound_var = ty::BoundVariableKind::Region(kind);
                bound_vars.push(bound_var);
                ty::Region::new_late_bound(
                    tcx,
                    ty::INNERMOST,
                    ty::BoundRegion { var: ty::BoundVar::from_usize(bound_vars.len() - 1), kind },
                )
                .into()
            }
            GenericParamDefKind::Const { .. } => {
                let bound_var = ty::BoundVariableKind::Const;
                bound_vars.push(bound_var);
                ty::Const::new_bound(
                    tcx,
                    ty::INNERMOST,
                    ty::BoundVar::from_usize(bound_vars.len() - 1),
                    tcx.type_of(param.def_id)
                        .no_bound_vars()
                        .expect("const parameter types cannot be generic"),
                )
                .into()
            }
        },
    );
    // When checking something like
    //
    // trait X { type Y: PartialEq<<Self as X>::Y> }
    // impl X for T { default type Y = S; }
    //
    // We will have to prove the bound S: PartialEq<<T as X>::Y>. In this case
    // we want <T as X>::Y to normalize to S. This is valid because we are
    // checking the default value specifically here. Add this equality to the
    // ParamEnv for normalization specifically.
    let normalize_impl_ty = tcx.type_of(impl_ty.def_id).instantiate(tcx, normalize_impl_ty_args);
    let rebased_args = normalize_impl_ty_args.rebase_onto(tcx, container_id, impl_trait_ref.args);
    let bound_vars = tcx.mk_bound_variable_kinds(&bound_vars);
    let normalize_param_env = {
        let mut predicates = param_env.caller_bounds().iter().collect::<Vec<_>>();
        match normalize_impl_ty.kind() {
            ty::Alias(ty::Projection, proj)
                if proj.def_id == trait_ty.def_id && proj.args == rebased_args =>
            {
                // Don't include this predicate if the projected type is
                // exactly the same as the projection. This can occur in
                // (somewhat dubious) code like this:
                //
                // impl<T> X for T where T: X { type Y = <T as X>::Y; }
            }
            _ => predicates.push(
                ty::Binder::bind_with_vars(
                    ty::ProjectionPredicate {
                        projection_ty: tcx.mk_alias_ty(trait_ty.def_id, rebased_args),
                        term: normalize_impl_ty.into(),
                    },
                    bound_vars,
                )
                .to_predicate(tcx),
            ),
        };
        ty::ParamEnv::new(tcx.mk_clauses(&predicates), Reveal::UserFacing)
    };
    debug!(?normalize_param_env);

    let impl_ty_def_id = impl_ty.def_id.expect_local();
    let impl_ty_args = GenericArgs::identity_for_item(tcx, impl_ty.def_id);
    let rebased_args = impl_ty_args.rebase_onto(tcx, container_id, impl_trait_ref.args);

    let infcx = tcx.infer_ctxt().build();
    let ocx = ObligationCtxt::new(&infcx);

    // A synthetic impl Trait for RPITIT desugaring has no HIR, which we currently use to get the
    // span for an impl's associated type. Instead, for these, use the def_span for the synthesized
    // associated type.
    let impl_ty_span = if impl_ty.is_impl_trait_in_trait() {
        tcx.def_span(impl_ty_def_id)
    } else {
        match tcx.hir().get_by_def_id(impl_ty_def_id) {
            hir::Node::TraitItem(hir::TraitItem {
                kind: hir::TraitItemKind::Type(_, Some(ty)),
                ..
            }) => ty.span,
            hir::Node::ImplItem(hir::ImplItem { kind: hir::ImplItemKind::Type(ty), .. }) => ty.span,
            _ => bug!(),
        }
    };
    let assumed_wf_types = ocx.assumed_wf_types_and_report_errors(param_env, impl_ty_def_id)?;

    let normalize_cause = ObligationCause::new(
        impl_ty_span,
        impl_ty_def_id,
        ObligationCauseCode::CheckAssociatedTypeBounds {
            impl_item_def_id: impl_ty.def_id.expect_local(),
            trait_item_def_id: trait_ty.def_id,
        },
    );
    let mk_cause = |span: Span| {
        let code = if span.is_dummy() {
            traits::ItemObligation(trait_ty.def_id)
        } else {
            traits::BindingObligation(trait_ty.def_id, span)
        };
        ObligationCause::new(impl_ty_span, impl_ty_def_id, code)
    };

    let obligations: Vec<_> = tcx
        .explicit_item_bounds(trait_ty.def_id)
        .iter_instantiated_copied(tcx, rebased_args)
        .map(|(concrete_ty_bound, span)| {
            debug!("check_type_bounds: concrete_ty_bound = {:?}", concrete_ty_bound);
            traits::Obligation::new(tcx, mk_cause(span), param_env, concrete_ty_bound)
        })
        .collect();
    debug!("check_type_bounds: item_bounds={:?}", obligations);

    for mut obligation in util::elaborate(tcx, obligations) {
        let normalized_predicate =
            ocx.normalize(&normalize_cause, normalize_param_env, obligation.predicate);
        debug!("compare_projection_bounds: normalized predicate = {:?}", normalized_predicate);
        obligation.predicate = normalized_predicate;

        ocx.register_obligation(obligation);
    }
    // Check that all obligations are satisfied by the implementation's
    // version.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        let reported = infcx.err_ctxt().report_fulfillment_errors(&errors);
        return Err(reported);
    }

    // Finally, resolve all regions. This catches wily misuses of
    // lifetime parameters.
    let implied_bounds = infcx.implied_bounds_tys(param_env, impl_ty_def_id, assumed_wf_types);
    let outlives_env = OutlivesEnvironment::with_bounds(param_env, implied_bounds);
    ocx.resolve_regions_and_report_errors(impl_ty_def_id, &outlives_env)
}

fn assoc_item_kind_str(impl_item: &ty::AssocItem) -> &'static str {
    match impl_item.kind {
        ty::AssocKind::Const => "const",
        ty::AssocKind::Fn => "method",
        ty::AssocKind::Type => "type",
    }
}
