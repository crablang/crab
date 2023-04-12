use crablangc_hir as hir;
use crablangc_hir::def::DefKind;
use crablangc_middle::ty::TyCtxt;
use crablangc_session::cstore::ForeignModule;

pub(crate) fn collect(tcx: TyCtxt<'_>) -> Vec<ForeignModule> {
    let mut modules = Vec::new();
    for id in tcx.hir().items() {
        if !matches!(tcx.def_kind(id.owner_id), DefKind::ForeignMod) {
            continue;
        }
        let item = tcx.hir().item(id);
        if let hir::ItemKind::ForeignMod { items, .. } = item.kind {
            let foreign_items = items.iter().map(|it| it.id.owner_id.to_def_id()).collect();
            modules.push(ForeignModule { foreign_items, def_id: id.owner_id.to_def_id() });
        }
    }
    modules
}
