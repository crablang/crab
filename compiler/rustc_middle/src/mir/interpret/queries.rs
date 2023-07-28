use super::{ErrorHandled, EvalToConstValueResult, EvalToValTreeResult, GlobalId};

use crate::mir;
use crate::query::{TyCtxtAt, TyCtxtEnsure};
use crate::ty::visit::TypeVisitableExt;
use crate::ty::GenericArgs;
use crate::ty::{self, TyCtxt};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_session::lint;
use rustc_span::{Span, DUMMY_SP};

impl<'tcx> TyCtxt<'tcx> {
    /// Evaluates a constant without providing any substitutions. This is useful to evaluate consts
    /// that can't take any generic arguments like statics, const items or enum discriminants. If a
    /// generic parameter is used within the constant `ErrorHandled::ToGeneric` will be returned.
    #[instrument(skip(self), level = "debug")]
    pub fn const_eval_poly(self, def_id: DefId) -> EvalToConstValueResult<'tcx> {
        // In some situations def_id will have substitutions within scope, but they aren't allowed
        // to be used. So we can't use `Instance::mono`, instead we feed unresolved substitutions
        // into `const_eval` which will return `ErrorHandled::ToGeneric` if any of them are
        // encountered.
        let args = GenericArgs::identity_for_item(self, def_id);
        let instance = ty::Instance::new(def_id, args);
        let cid = GlobalId { instance, promoted: None };
        let param_env = self.param_env(def_id).with_reveal_all_normalized(self);
        self.const_eval_global_id(param_env, cid, None)
    }
    /// Resolves and evaluates a constant.
    ///
    /// The constant can be located on a trait like `<A as B>::C`, in which case the given
    /// substitutions and environment are used to resolve the constant. Alternatively if the
    /// constant has generic parameters in scope the substitutions are used to evaluate the value of
    /// the constant. For example in `fn foo<T>() { let _ = [0; bar::<T>()]; }` the repeat count
    /// constant `bar::<T>()` requires a substitution for `T`, if the substitution for `T` is still
    /// too generic for the constant to be evaluated then `Err(ErrorHandled::TooGeneric)` is
    /// returned.
    #[instrument(level = "debug", skip(self))]
    pub fn const_eval_resolve(
        self,
        param_env: ty::ParamEnv<'tcx>,
        ct: mir::UnevaluatedConst<'tcx>,
        span: Option<Span>,
    ) -> EvalToConstValueResult<'tcx> {
        // Cannot resolve `Unevaluated` constants that contain inference
        // variables. We reject those here since `resolve`
        // would fail otherwise.
        //
        // When trying to evaluate constants containing inference variables,
        // use `Infcx::const_eval_resolve` instead.
        if ct.args.has_non_region_infer() {
            bug!("did not expect inference variables here");
        }

        match ty::Instance::resolve(
            self, param_env,
            // FIXME: maybe have a separate version for resolving mir::UnevaluatedConst?
            ct.def, ct.args,
        ) {
            Ok(Some(instance)) => {
                let cid = GlobalId { instance, promoted: ct.promoted };
                self.const_eval_global_id(param_env, cid, span)
            }
            Ok(None) => Err(ErrorHandled::TooGeneric),
            Err(err) => Err(ErrorHandled::Reported(err.into())),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn const_eval_resolve_for_typeck(
        self,
        param_env: ty::ParamEnv<'tcx>,
        ct: ty::UnevaluatedConst<'tcx>,
        span: Option<Span>,
    ) -> EvalToValTreeResult<'tcx> {
        // Cannot resolve `Unevaluated` constants that contain inference
        // variables. We reject those here since `resolve`
        // would fail otherwise.
        //
        // When trying to evaluate constants containing inference variables,
        // use `Infcx::const_eval_resolve` instead.
        if ct.args.has_non_region_infer() {
            bug!("did not expect inference variables here");
        }

        match ty::Instance::resolve(self, param_env, ct.def, ct.args) {
            Ok(Some(instance)) => {
                let cid = GlobalId { instance, promoted: None };
                self.const_eval_global_id_for_typeck(param_env, cid, span).inspect(|_| {
                    // We are emitting the lint here instead of in `is_const_evaluatable`
                    // as we normalize obligations before checking them, and normalization
                    // uses this function to evaluate this constant.
                    //
                    // @lcnr believes that successfully evaluating even though there are
                    // used generic parameters is a bug of evaluation, so checking for it
                    // here does feel somewhat sensible.
                    if !self.features().generic_const_exprs && ct.args.has_non_region_param() {
                        let def_kind = self.def_kind(instance.def_id());
                        assert!(
                            matches!(
                                def_kind,
                                DefKind::InlineConst | DefKind::AnonConst | DefKind::AssocConst
                            ),
                            "{cid:?} is {def_kind:?}",
                        );
                        let mir_body = self.mir_for_ctfe(instance.def_id());
                        if mir_body.is_polymorphic {
                            let Some(local_def_id) = ct.def.as_local() else { return };
                            self.struct_span_lint_hir(
                                lint::builtin::CONST_EVALUATABLE_UNCHECKED,
                                self.hir().local_def_id_to_hir_id(local_def_id),
                                self.def_span(ct.def),
                                "cannot use constants which depend on generic parameters in types",
                                |err| err,
                            )
                        }
                    }
                })
            }
            Ok(None) => Err(ErrorHandled::TooGeneric),
            Err(err) => Err(ErrorHandled::Reported(err.into())),
        }
    }

    pub fn const_eval_instance(
        self,
        param_env: ty::ParamEnv<'tcx>,
        instance: ty::Instance<'tcx>,
        span: Option<Span>,
    ) -> EvalToConstValueResult<'tcx> {
        self.const_eval_global_id(param_env, GlobalId { instance, promoted: None }, span)
    }

    /// Evaluate a constant to a `ConstValue`.
    #[instrument(skip(self), level = "debug")]
    pub fn const_eval_global_id(
        self,
        param_env: ty::ParamEnv<'tcx>,
        cid: GlobalId<'tcx>,
        span: Option<Span>,
    ) -> EvalToConstValueResult<'tcx> {
        // Const-eval shouldn't depend on lifetimes at all, so we can erase them, which should
        // improve caching of queries.
        let inputs = self.erase_regions(param_env.and(cid));
        if let Some(span) = span {
            self.at(span).eval_to_const_value_raw(inputs)
        } else {
            self.eval_to_const_value_raw(inputs)
        }
    }

    /// Evaluate a constant to a type-level constant.
    #[instrument(skip(self), level = "debug")]
    pub fn const_eval_global_id_for_typeck(
        self,
        param_env: ty::ParamEnv<'tcx>,
        cid: GlobalId<'tcx>,
        span: Option<Span>,
    ) -> EvalToValTreeResult<'tcx> {
        // Const-eval shouldn't depend on lifetimes at all, so we can erase them, which should
        // improve caching of queries.
        let inputs = self.erase_regions(param_env.and(cid));
        debug!(?inputs);
        if let Some(span) = span {
            self.at(span).eval_to_valtree(inputs)
        } else {
            self.eval_to_valtree(inputs)
        }
    }

    /// Evaluate a static's initializer, returning the allocation of the initializer's memory.
    #[inline(always)]
    pub fn eval_static_initializer(
        self,
        def_id: DefId,
    ) -> Result<mir::ConstAllocation<'tcx>, ErrorHandled> {
        self.at(DUMMY_SP).eval_static_initializer(def_id)
    }
}

impl<'tcx> TyCtxtAt<'tcx> {
    /// Evaluate a static's initializer, returning the allocation of the initializer's memory.
    ///
    /// The span is entirely ignored here, but still helpful for better query cycle errors.
    pub fn eval_static_initializer(
        self,
        def_id: DefId,
    ) -> Result<mir::ConstAllocation<'tcx>, ErrorHandled> {
        trace!("eval_static_initializer: Need to compute {:?}", def_id);
        assert!(self.is_static(def_id));
        let instance = ty::Instance::mono(*self, def_id);
        let gid = GlobalId { instance, promoted: None };
        self.eval_to_allocation(gid, ty::ParamEnv::reveal_all())
    }

    /// Evaluate anything constant-like, returning the allocation of the final memory.
    ///
    /// The span is entirely ignored here, but still helpful for better query cycle errors.
    fn eval_to_allocation(
        self,
        gid: GlobalId<'tcx>,
        param_env: ty::ParamEnv<'tcx>,
    ) -> Result<mir::ConstAllocation<'tcx>, ErrorHandled> {
        trace!("eval_to_allocation: Need to compute {:?}", gid);
        let raw_const = self.eval_to_allocation_raw(param_env.and(gid))?;
        Ok(self.global_alloc(raw_const.alloc_id).unwrap_memory())
    }
}

impl<'tcx> TyCtxtEnsure<'tcx> {
    /// Evaluates a constant without providing any substitutions. This is useful to evaluate consts
    /// that can't take any generic arguments like statics, const items or enum discriminants. If a
    /// generic parameter is used within the constant `ErrorHandled::ToGeneric` will be returned.
    #[instrument(skip(self), level = "debug")]
    pub fn const_eval_poly(self, def_id: DefId) {
        // In some situations def_id will have substitutions within scope, but they aren't allowed
        // to be used. So we can't use `Instance::mono`, instead we feed unresolved substitutions
        // into `const_eval` which will return `ErrorHandled::ToGeneric` if any of them are
        // encountered.
        let args = GenericArgs::identity_for_item(self.tcx, def_id);
        let instance = ty::Instance::new(def_id, args);
        let cid = GlobalId { instance, promoted: None };
        let param_env = self.tcx.param_env(def_id).with_reveal_all_normalized(self.tcx);
        // Const-eval shouldn't depend on lifetimes at all, so we can erase them, which should
        // improve caching of queries.
        let inputs = self.tcx.erase_regions(param_env.and(cid));
        self.eval_to_const_value_raw(inputs)
    }

    /// Evaluate a static's initializer, returning the allocation of the initializer's memory.
    pub fn eval_static_initializer(self, def_id: DefId) {
        trace!("eval_static_initializer: Need to compute {:?}", def_id);
        assert!(self.tcx.is_static(def_id));
        let instance = ty::Instance::mono(self.tcx, def_id);
        let gid = GlobalId { instance, promoted: None };
        let param_env = ty::ParamEnv::reveal_all();
        trace!("eval_to_allocation: Need to compute {:?}", gid);
        self.eval_to_allocation_raw(param_env.and(gid))
    }
}
