use crate::infer::canonical::{Canonical, CanonicalQueryResponse};
use crate::traits::query::Fallible;
use crablangc_middle::ty::fold::TypeFoldable;
use crablangc_middle::ty::{self, Lift, ParamEnvAnd, Ty, TyCtxt, TypeVisitableExt};
use std::fmt;

pub use crablangc_middle::traits::query::type_op::Normalize;

impl<'tcx, T> super::QueryTypeOp<'tcx> for Normalize<T>
where
    T: Normalizable<'tcx> + 'tcx,
{
    type QueryResponse = T;

    fn try_fast_path(_tcx: TyCtxt<'tcx>, key: &ParamEnvAnd<'tcx, Self>) -> Option<T> {
        if !key.value.value.has_projections() { Some(key.value.value) } else { None }
    }

    fn perform_query(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Self>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self::QueryResponse>> {
        T::type_op_method(tcx, canonicalized)
    }
}

pub trait Normalizable<'tcx>: fmt::Debug + TypeFoldable<TyCtxt<'tcx>> + Lift<'tcx> + Copy {
    fn type_op_method(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Normalize<Self>>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self>>;
}

impl<'tcx> Normalizable<'tcx> for Ty<'tcx> {
    fn type_op_method(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Normalize<Self>>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self>> {
        tcx.type_op_normalize_ty(canonicalized)
    }
}

impl<'tcx> Normalizable<'tcx> for ty::Predicate<'tcx> {
    fn type_op_method(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Normalize<Self>>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self>> {
        tcx.type_op_normalize_predicate(canonicalized)
    }
}

impl<'tcx> Normalizable<'tcx> for ty::PolyFnSig<'tcx> {
    fn type_op_method(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Normalize<Self>>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self>> {
        tcx.type_op_normalize_poly_fn_sig(canonicalized)
    }
}

impl<'tcx> Normalizable<'tcx> for ty::FnSig<'tcx> {
    fn type_op_method(
        tcx: TyCtxt<'tcx>,
        canonicalized: Canonical<'tcx, ParamEnvAnd<'tcx, Normalize<Self>>>,
    ) -> Fallible<CanonicalQueryResponse<'tcx, Self>> {
        tcx.type_op_normalize_fn_sig(canonicalized)
    }
}
