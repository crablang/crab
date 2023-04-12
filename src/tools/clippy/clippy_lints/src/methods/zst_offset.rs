use clippy_utils::diagnostics::span_lint;
use if_chain::if_chain;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_middle::ty;

use super::ZST_OFFSET;

pub(super) fn check(cx: &LateContext<'_>, expr: &hir::Expr<'_>, recv: &hir::Expr<'_>) {
    if_chain! {
        if let ty::RawPtr(ty::TypeAndMut { ty, .. }) = cx.typeck_results().expr_ty(recv).kind();
        if let Ok(layout) = cx.tcx.layout_of(cx.param_env.and(*ty));
        if layout.is_zst();
        then {
            span_lint(cx, ZST_OFFSET, expr.span, "offset calculation on zero-sized value");
        }
    }
}
