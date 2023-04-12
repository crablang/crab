use super::BackendTypes;
use crablangc_hir::def_id::DefId;
use crablangc_middle::mir::mono::{Linkage, Visibility};
use crablangc_middle::ty::Instance;

pub trait PreDefineMethods<'tcx>: BackendTypes {
    fn predefine_static(
        &self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    );
    fn predefine_fn(
        &self,
        instance: Instance<'tcx>,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    );
}
