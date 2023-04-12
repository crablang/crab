use crablangc_ast::attr;
use crablangc_hir::def_id::LocalDefId;
use crablangc_middle::ty::query::Providers;
use crablangc_middle::ty::TyCtxt;
use crablangc_span::symbol::sym;

fn proc_macro_decls_static(tcx: TyCtxt<'_>, (): ()) -> Option<LocalDefId> {
    let mut decls = None;

    for id in tcx.hir().items() {
        let attrs = tcx.hir().attrs(id.hir_id());
        if attr::contains_name(attrs, sym::crablangc_proc_macro_decls) {
            decls = Some(id.owner_id.def_id);
        }
    }

    decls
}

pub(crate) fn provide(providers: &mut Providers) {
    *providers = Providers { proc_macro_decls_static, ..*providers };
}
