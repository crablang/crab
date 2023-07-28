use crate::traits::query::evaluate_obligation::InferCtxtExt as _;
use crate::traits::{self, DefiningAnchor, ObligationCtxt};

use rustc_hir::def_id::DefId;
use rustc_hir::lang_items::LangItem;
use rustc_middle::arena::ArenaAllocatable;
use rustc_middle::infer::canonical::{Canonical, CanonicalQueryResponse, QueryResponse};
use rustc_middle::traits::query::NoSolution;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeFoldable, TypeVisitableExt};
use rustc_middle::ty::{GenericArg, ToPredicate};
use rustc_span::DUMMY_SP;

use std::fmt::Debug;

pub use rustc_infer::infer::*;

pub trait InferCtxtExt<'tcx> {
    fn type_is_copy_modulo_regions(&self, param_env: ty::ParamEnv<'tcx>, ty: Ty<'tcx>) -> bool;

    fn type_is_sized_modulo_regions(&self, param_env: ty::ParamEnv<'tcx>, ty: Ty<'tcx>) -> bool;

    /// Check whether a `ty` implements given trait(trait_def_id).
    /// The inputs are:
    ///
    /// - the def-id of the trait
    /// - the type parameters of the trait, including the self-type
    /// - the parameter environment
    ///
    /// Invokes `evaluate_obligation`, so in the event that evaluating
    /// `Ty: Trait` causes overflow, EvaluatedToRecur (or EvaluatedToUnknown)
    /// will be returned.
    fn type_implements_trait(
        &self,
        trait_def_id: DefId,
        params: impl IntoIterator<Item: Into<GenericArg<'tcx>>>,
        param_env: ty::ParamEnv<'tcx>,
    ) -> traits::EvaluationResult;
}

impl<'tcx> InferCtxtExt<'tcx> for InferCtxt<'tcx> {
    fn type_is_copy_modulo_regions(&self, param_env: ty::ParamEnv<'tcx>, ty: Ty<'tcx>) -> bool {
        let ty = self.resolve_vars_if_possible(ty);

        if !(param_env, ty).has_infer() {
            return ty.is_copy_modulo_regions(self.tcx, param_env);
        }

        let copy_def_id = self.tcx.require_lang_item(LangItem::Copy, None);

        // This can get called from typeck (by euv), and `moves_by_default`
        // rightly refuses to work with inference variables, but
        // moves_by_default has a cache, which we want to use in other
        // cases.
        traits::type_known_to_meet_bound_modulo_regions(self, param_env, ty, copy_def_id)
    }

    fn type_is_sized_modulo_regions(&self, param_env: ty::ParamEnv<'tcx>, ty: Ty<'tcx>) -> bool {
        let lang_item = self.tcx.require_lang_item(LangItem::Sized, None);
        traits::type_known_to_meet_bound_modulo_regions(self, param_env, ty, lang_item)
    }

    #[instrument(level = "debug", skip(self, params), ret)]
    fn type_implements_trait(
        &self,
        trait_def_id: DefId,
        params: impl IntoIterator<Item: Into<GenericArg<'tcx>>>,
        param_env: ty::ParamEnv<'tcx>,
    ) -> traits::EvaluationResult {
        let trait_ref = ty::TraitRef::new(self.tcx, trait_def_id, params);

        let obligation = traits::Obligation {
            cause: traits::ObligationCause::dummy(),
            param_env,
            recursion_depth: 0,
            predicate: ty::Binder::dummy(trait_ref).without_const().to_predicate(self.tcx),
        };
        self.evaluate_obligation(&obligation).unwrap_or(traits::EvaluationResult::EvaluatedToErr)
    }
}

pub trait InferCtxtBuilderExt<'tcx> {
    fn enter_canonical_trait_query<K, R>(
        self,
        canonical_key: &Canonical<'tcx, K>,
        operation: impl FnOnce(&ObligationCtxt<'_, 'tcx>, K) -> Result<R, NoSolution>,
    ) -> Result<CanonicalQueryResponse<'tcx, R>, NoSolution>
    where
        K: TypeFoldable<TyCtxt<'tcx>>,
        R: Debug + TypeFoldable<TyCtxt<'tcx>>,
        Canonical<'tcx, QueryResponse<'tcx, R>>: ArenaAllocatable<'tcx>;
}

impl<'tcx> InferCtxtBuilderExt<'tcx> for InferCtxtBuilder<'tcx> {
    /// The "main method" for a canonicalized trait query. Given the
    /// canonical key `canonical_key`, this method will create a new
    /// inference context, instantiate the key, and run your operation
    /// `op`. The operation should yield up a result (of type `R`) as
    /// well as a set of trait obligations that must be fully
    /// satisfied. These obligations will be processed and the
    /// canonical result created.
    ///
    /// Returns `NoSolution` in the event of any error.
    ///
    /// (It might be mildly nicer to implement this on `TyCtxt`, and
    /// not `InferCtxtBuilder`, but that is a bit tricky right now.
    /// In part because we would need a `for<'tcx>` sort of
    /// bound for the closure and in part because it is convenient to
    /// have `'tcx` be free on this function so that we can talk about
    /// `K: TypeFoldable<TyCtxt<'tcx>>`.)
    fn enter_canonical_trait_query<K, R>(
        self,
        canonical_key: &Canonical<'tcx, K>,
        operation: impl FnOnce(&ObligationCtxt<'_, 'tcx>, K) -> Result<R, NoSolution>,
    ) -> Result<CanonicalQueryResponse<'tcx, R>, NoSolution>
    where
        K: TypeFoldable<TyCtxt<'tcx>>,
        R: Debug + TypeFoldable<TyCtxt<'tcx>>,
        Canonical<'tcx, QueryResponse<'tcx, R>>: ArenaAllocatable<'tcx>,
    {
        let (infcx, key, canonical_inference_vars) = self
            .with_opaque_type_inference(DefiningAnchor::Bubble)
            .build_with_canonical(DUMMY_SP, canonical_key);
        let ocx = ObligationCtxt::new(&infcx);
        let value = operation(&ocx, key)?;
        ocx.make_canonicalized_query_response(canonical_inference_vars, value)
    }
}
