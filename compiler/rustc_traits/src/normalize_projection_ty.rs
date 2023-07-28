use rustc_infer::infer::canonical::{Canonical, QueryResponse};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::query::Providers;
use rustc_middle::ty::{ParamEnvAnd, TyCtxt};
use rustc_trait_selection::infer::InferCtxtBuilderExt;
use rustc_trait_selection::traits::query::{
    normalize::NormalizationResult, CanonicalProjectionGoal, NoSolution,
};
use rustc_trait_selection::traits::{self, ObligationCause, SelectionContext};
use std::sync::atomic::Ordering;

pub(crate) fn provide(p: &mut Providers) {
    *p = Providers {
        normalize_projection_ty,
        normalize_weak_ty,
        normalize_inherent_projection_ty,
        ..*p
    };
}

fn normalize_projection_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    goal: CanonicalProjectionGoal<'tcx>,
) -> Result<&'tcx Canonical<'tcx, QueryResponse<'tcx, NormalizationResult<'tcx>>>, NoSolution> {
    debug!("normalize_provider(goal={:#?})", goal);

    tcx.sess.perf_stats.normalize_projection_ty.fetch_add(1, Ordering::Relaxed);
    tcx.infer_ctxt().enter_canonical_trait_query(
        &goal,
        |ocx, ParamEnvAnd { param_env, value: goal }| {
            let selcx = &mut SelectionContext::new(ocx.infcx);
            let cause = ObligationCause::dummy();
            let mut obligations = vec![];
            let answer = traits::normalize_projection_type(
                selcx,
                param_env,
                goal,
                cause,
                0,
                &mut obligations,
            );
            ocx.register_obligations(obligations);
            // FIXME(associated_const_equality): All users of normalize_projection_ty expected
            // a type, but there is the possibility it could've been a const now. Maybe change
            // it to a Term later?
            Ok(NormalizationResult { normalized_ty: answer.ty().unwrap() })
        },
    )
}

fn normalize_weak_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    goal: CanonicalProjectionGoal<'tcx>,
) -> Result<&'tcx Canonical<'tcx, QueryResponse<'tcx, NormalizationResult<'tcx>>>, NoSolution> {
    debug!("normalize_provider(goal={:#?})", goal);

    tcx.sess.perf_stats.normalize_projection_ty.fetch_add(1, Ordering::Relaxed);
    tcx.infer_ctxt().enter_canonical_trait_query(
        &goal,
        |ocx, ParamEnvAnd { param_env, value: goal }| {
            let obligations = tcx.predicates_of(goal.def_id).instantiate_own(tcx, goal.args).map(
                |(predicate, span)| {
                    traits::Obligation::new(
                        tcx,
                        ObligationCause::dummy_with_span(span),
                        param_env,
                        predicate,
                    )
                },
            );
            ocx.register_obligations(obligations);
            let normalized_ty = tcx.type_of(goal.def_id).instantiate(tcx, goal.args);
            Ok(NormalizationResult { normalized_ty })
        },
    )
}

fn normalize_inherent_projection_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    goal: CanonicalProjectionGoal<'tcx>,
) -> Result<&'tcx Canonical<'tcx, QueryResponse<'tcx, NormalizationResult<'tcx>>>, NoSolution> {
    debug!("normalize_provider(goal={:#?})", goal);

    tcx.infer_ctxt().enter_canonical_trait_query(
        &goal,
        |ocx, ParamEnvAnd { param_env, value: goal }| {
            let selcx = &mut SelectionContext::new(ocx.infcx);
            let cause = ObligationCause::dummy();
            let mut obligations = vec![];
            let answer = traits::normalize_inherent_projection(
                selcx,
                param_env,
                goal,
                cause,
                0,
                &mut obligations,
            );
            ocx.register_obligations(obligations);

            Ok(NormalizationResult { normalized_ty: answer })
        },
    )
}
