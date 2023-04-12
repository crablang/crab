use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::{path_def_id, qpath_generic_tys};
use if_chain::if_chain;
use crablangc_hir::{self as hir, def_id::DefId, QPath};
use crablangc_lint::LateContext;
use crablangc_span::symbol::sym;

use super::RC_MUTEX;

pub(super) fn check(cx: &LateContext<'_>, hir_ty: &hir::Ty<'_>, qpath: &QPath<'_>, def_id: DefId) -> bool {
    if_chain! {
        if cx.tcx.is_diagnostic_item(sym::Rc, def_id) ;
        if let Some(arg) = qpath_generic_tys(qpath).next();
        if let Some(id) = path_def_id(cx, arg);
        if cx.tcx.is_diagnostic_item(sym::Mutex, id);
        then {
            span_lint_and_help(
                cx,
                RC_MUTEX,
                hir_ty.span,
                "usage of `Rc<Mutex<_>>`",
                None,
                "consider using `Rc<RefCell<_>>` or `Arc<Mutex<_>>` instead",
            );
            return true;
        }
    }

    false
}
