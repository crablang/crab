//! The new trait solver, currently still WIP.
//!
//! As a user of the trait system, you can use `TyCtxt::evaluate_goal` to
//! interact with this solver.
//!
//! For a high-level overview of how this solver works, check out the relevant
//! section of the crablangc-dev-guide.
//!
//! FIXME(@lcnr): Write that section. If you read this before then ask me
//! about it on zulip.

use crablangc_hir::def_id::DefId;
use crablangc_infer::infer::canonical::{Canonical, CanonicalVarValues};
use crablangc_infer::traits::query::NoSolution;
use crablangc_middle::traits::solve::{
    CanonicalResponse, Certainty, ExternalConstraintsData, Goal, QueryResult, Response,
};
use crablangc_middle::ty::{self, Ty, TyCtxt};
use crablangc_middle::ty::{
    CoercePredicate, RegionOutlivesPredicate, SubtypePredicate, TypeOutlivesPredicate,
};

mod assembly;
mod canonicalize;
mod eval_ctxt;
mod fulfill;
mod project_goals;
mod search_graph;
mod trait_goals;

pub use eval_ctxt::{EvalCtxt, InferCtxtEvalExt};
pub use fulfill::FulfillmentCtxt;

#[derive(Debug, Clone, Copy)]
enum SolverMode {
    /// Ordinary trait solving, using everywhere except for coherence.
    Normal,
    /// Trait solving during coherence. There are a few notable differences
    /// between coherence and ordinary trait solving.
    ///
    /// Most importantly, trait solving during coherence must not be incomplete,
    /// i.e. return `Err(NoSolution)` for goals for which a solution exists.
    /// This means that we must not make any guesses or arbitrary choices.
    Coherence,
}

trait CanonicalResponseExt {
    fn has_no_inference_or_external_constraints(&self) -> bool;

    fn has_only_region_constraints(&self) -> bool;
}

impl<'tcx> CanonicalResponseExt for Canonical<'tcx, Response<'tcx>> {
    fn has_no_inference_or_external_constraints(&self) -> bool {
        self.value.external_constraints.region_constraints.is_empty()
            && self.value.var_values.is_identity()
            && self.value.external_constraints.opaque_types.is_empty()
    }

    fn has_only_region_constraints(&self) -> bool {
        self.value.var_values.is_identity_modulo_regions()
            && self.value.external_constraints.opaque_types.is_empty()
    }
}

impl<'a, 'tcx> EvalCtxt<'a, 'tcx> {
    #[instrument(level = "debug", skip(self))]
    fn compute_type_outlives_goal(
        &mut self,
        goal: Goal<'tcx, TypeOutlivesPredicate<'tcx>>,
    ) -> QueryResult<'tcx> {
        let ty::OutlivesPredicate(ty, lt) = goal.predicate;
        self.register_ty_outlives(ty, lt);
        self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_region_outlives_goal(
        &mut self,
        goal: Goal<'tcx, RegionOutlivesPredicate<'tcx>>,
    ) -> QueryResult<'tcx> {
        let ty::OutlivesPredicate(a, b) = goal.predicate;
        self.register_region_outlives(a, b);
        self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_coerce_goal(
        &mut self,
        goal: Goal<'tcx, CoercePredicate<'tcx>>,
    ) -> QueryResult<'tcx> {
        self.compute_subtype_goal(Goal {
            param_env: goal.param_env,
            predicate: SubtypePredicate {
                a_is_expected: false,
                a: goal.predicate.a,
                b: goal.predicate.b,
            },
        })
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_subtype_goal(
        &mut self,
        goal: Goal<'tcx, SubtypePredicate<'tcx>>,
    ) -> QueryResult<'tcx> {
        if goal.predicate.a.is_ty_var() && goal.predicate.b.is_ty_var() {
            self.evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS)
        } else {
            self.sub(goal.param_env, goal.predicate.a, goal.predicate.b)?;
            self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_closure_kind_goal(
        &mut self,
        goal: Goal<'tcx, (DefId, ty::SubstsRef<'tcx>, ty::ClosureKind)>,
    ) -> QueryResult<'tcx> {
        let (_, substs, expected_kind) = goal.predicate;
        let found_kind = substs.as_closure().kind_ty().to_opt_closure_kind();

        let Some(found_kind) = found_kind else {
            return self.evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS);
        };
        if found_kind.extends(expected_kind) {
            self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        } else {
            Err(NoSolution)
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_object_safe_goal(&mut self, trait_def_id: DefId) -> QueryResult<'tcx> {
        if self.tcx().check_is_object_safe(trait_def_id) {
            self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
        } else {
            Err(NoSolution)
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn compute_well_formed_goal(
        &mut self,
        goal: Goal<'tcx, ty::GenericArg<'tcx>>,
    ) -> QueryResult<'tcx> {
        match self.well_formed_goals(goal.param_env, goal.predicate) {
            Some(goals) => {
                self.add_goals(goals);
                self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
            }
            None => self.evaluate_added_goals_and_make_canonical_response(Certainty::AMBIGUOUS),
        }
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn compute_alias_relate_goal(
        &mut self,
        goal: Goal<'tcx, (ty::Term<'tcx>, ty::Term<'tcx>, ty::AliasRelationDirection)>,
    ) -> QueryResult<'tcx> {
        let tcx = self.tcx();
        // We may need to invert the alias relation direction if dealing an alias on the RHS.
        #[derive(Debug)]
        enum Invert {
            No,
            Yes,
        }
        let evaluate_normalizes_to =
            |ecx: &mut EvalCtxt<'_, 'tcx>, alias, other, direction, invert| {
                let span = tracing::span!(
                    tracing::Level::DEBUG,
                    "compute_alias_relate_goal(evaluate_normalizes_to)",
                    ?alias,
                    ?other,
                    ?direction,
                    ?invert
                );
                let _enter = span.enter();
                let result = ecx.probe(|ecx| {
                    let other = match direction {
                        // This is purely an optimization.
                        ty::AliasRelationDirection::Equate => other,

                        ty::AliasRelationDirection::Subtype => {
                            let fresh = ecx.next_term_infer_of_kind(other);
                            let (sub, sup) = match invert {
                                Invert::No => (fresh, other),
                                Invert::Yes => (other, fresh),
                            };
                            ecx.sub(goal.param_env, sub, sup)?;
                            fresh
                        }
                    };
                    ecx.add_goal(goal.with(
                        tcx,
                        ty::Binder::dummy(ty::ProjectionPredicate {
                            projection_ty: alias,
                            term: other,
                        }),
                    ));
                    ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                });
                debug!(?result);
                result
            };

        let (lhs, rhs, direction) = goal.predicate;

        if lhs.is_infer() || rhs.is_infer() {
            bug!(
                "`AliasRelate` goal with an infer var on lhs or rhs which should have been instantiated"
            );
        }

        match (lhs.to_projection_term(tcx), rhs.to_projection_term(tcx)) {
            (None, None) => bug!("`AliasRelate` goal without an alias on either lhs or rhs"),

            // RHS is not a projection, only way this is true is if LHS normalizes-to RHS
            (Some(alias_lhs), None) => {
                evaluate_normalizes_to(self, alias_lhs, rhs, direction, Invert::No)
            }

            // LHS is not a projection, only way this is true is if RHS normalizes-to LHS
            (None, Some(alias_rhs)) => {
                evaluate_normalizes_to(self, alias_rhs, lhs, direction, Invert::Yes)
            }

            (Some(alias_lhs), Some(alias_rhs)) => {
                debug!("both sides are aliases");

                let mut candidates = Vec::new();
                // LHS normalizes-to RHS
                candidates.extend(
                    evaluate_normalizes_to(self, alias_lhs, rhs, direction, Invert::No).ok(),
                );
                // RHS normalizes-to RHS
                candidates.extend(
                    evaluate_normalizes_to(self, alias_rhs, lhs, direction, Invert::Yes).ok(),
                );
                // Relate via substs
                candidates.extend(
                    self.probe(|ecx| {
                        let span = tracing::span!(
                            tracing::Level::DEBUG,
                            "compute_alias_relate_goal(relate_via_substs)",
                            ?alias_lhs,
                            ?alias_rhs,
                            ?direction
                        );
                        let _enter = span.enter();

                        match direction {
                            ty::AliasRelationDirection::Equate => {
                                ecx.eq(goal.param_env, alias_lhs, alias_rhs)?;
                            }
                            ty::AliasRelationDirection::Subtype => {
                                ecx.sub(goal.param_env, alias_lhs, alias_rhs)?;
                            }
                        }

                        ecx.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
                    })
                    .ok(),
                );
                debug!(?candidates);

                if let Some(merged) = self.try_merge_responses(&candidates) {
                    Ok(merged)
                } else {
                    self.flounder(&candidates)
                }
            }
        }
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn compute_const_arg_has_type_goal(
        &mut self,
        goal: Goal<'tcx, (ty::Const<'tcx>, Ty<'tcx>)>,
    ) -> QueryResult<'tcx> {
        let (ct, ty) = goal.predicate;
        self.eq(goal.param_env, ct.ty(), ty)?;
        self.evaluate_added_goals_and_make_canonical_response(Certainty::Yes)
    }
}

impl<'tcx> EvalCtxt<'_, 'tcx> {
    #[instrument(level = "debug", skip(self))]
    fn set_normalizes_to_hack_goal(&mut self, goal: Goal<'tcx, ty::ProjectionPredicate<'tcx>>) {
        assert!(
            self.nested_goals.normalizes_to_hack_goal.is_none(),
            "attempted to set the projection eq hack goal when one already exists"
        );
        self.nested_goals.normalizes_to_hack_goal = Some(goal);
    }

    #[instrument(level = "debug", skip(self))]
    fn add_goal(&mut self, goal: Goal<'tcx, ty::Predicate<'tcx>>) {
        self.nested_goals.goals.push(goal);
    }

    #[instrument(level = "debug", skip(self, goals))]
    fn add_goals(&mut self, goals: impl IntoIterator<Item = Goal<'tcx, ty::Predicate<'tcx>>>) {
        let current_len = self.nested_goals.goals.len();
        self.nested_goals.goals.extend(goals);
        debug!("added_goals={:?}", &self.nested_goals.goals[current_len..]);
    }

    /// Try to merge multiple possible ways to prove a goal, if that is not possible returns `None`.
    ///
    /// In this case we tend to flounder and return ambiguity by calling `[EvalCtxt::flounder]`.
    #[instrument(level = "debug", skip(self), ret)]
    fn try_merge_responses(
        &mut self,
        responses: &[CanonicalResponse<'tcx>],
    ) -> Option<CanonicalResponse<'tcx>> {
        if responses.is_empty() {
            return None;
        }

        // FIXME(-Ztrait-solver=next): We should instead try to find a `Certainty::Yes` response with
        // a subset of the constraints that all the other responses have.
        let one = responses[0];
        if responses[1..].iter().all(|&resp| resp == one) {
            return Some(one);
        }

        responses
            .iter()
            .find(|response| {
                response.value.certainty == Certainty::Yes
                    && response.has_no_inference_or_external_constraints()
            })
            .copied()
    }

    /// If we fail to merge responses we flounder and return overflow or ambiguity.
    #[instrument(level = "debug", skip(self), ret)]
    fn flounder(&mut self, responses: &[CanonicalResponse<'tcx>]) -> QueryResult<'tcx> {
        if responses.is_empty() {
            return Err(NoSolution);
        }
        let certainty = responses.iter().fold(Certainty::AMBIGUOUS, |certainty, response| {
            certainty.unify_with(response.value.certainty)
        });

        let response = self.evaluate_added_goals_and_make_canonical_response(certainty);
        if let Ok(response) = response {
            assert!(response.has_no_inference_or_external_constraints());
            Ok(response)
        } else {
            bug!("failed to make floundered response: {responses:?}");
        }
    }
}

pub(super) fn response_no_constraints<'tcx>(
    tcx: TyCtxt<'tcx>,
    goal: Canonical<'tcx, impl Sized>,
    certainty: Certainty,
) -> QueryResult<'tcx> {
    Ok(Canonical {
        max_universe: goal.max_universe,
        variables: goal.variables,
        value: Response {
            var_values: CanonicalVarValues::make_identity(tcx, goal.variables),
            // FIXME: maybe we should store the "no response" version in tcx, like
            // we do for tcx.types and stuff.
            external_constraints: tcx.mk_external_constraints(ExternalConstraintsData::default()),
            certainty,
        },
    })
}
