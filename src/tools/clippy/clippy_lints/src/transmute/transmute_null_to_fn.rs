use clippy_utils::consts::{constant, Constant};
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::{is_integer_literal, is_path_diagnostic_item};
use crablangc_hir::{Expr, ExprKind};
use crablangc_lint::LateContext;
use crablangc_middle::ty::Ty;
use crablangc_span::symbol::sym;

use super::TRANSMUTE_NULL_TO_FN;

fn lint_expr(cx: &LateContext<'_>, expr: &Expr<'_>) {
    span_lint_and_then(
        cx,
        TRANSMUTE_NULL_TO_FN,
        expr.span,
        "transmuting a known null pointer into a function pointer",
        |diag| {
            diag.span_label(expr.span, "this transmute results in undefined behavior");
            diag.help(
               "try wrapping your function pointer type in `Option<T>` instead, and using `None` as a null pointer value"
            );
        },
    );
}

pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>, arg: &'tcx Expr<'_>, to_ty: Ty<'tcx>) -> bool {
    if !to_ty.is_fn() {
        return false;
    }

    match arg.kind {
        // Catching:
        // transmute over constants that resolve to `null`.
        ExprKind::Path(ref _qpath)
            if matches!(constant(cx, cx.typeck_results(), arg), Some((Constant::RawPtr(0), _))) =>
        {
            lint_expr(cx, expr);
            true
        },

        // Catching:
        // `std::mem::transmute(0 as *const i32)`
        ExprKind::Cast(inner_expr, _cast_ty) if is_integer_literal(inner_expr, 0) => {
            lint_expr(cx, expr);
            true
        },

        // Catching:
        // `std::mem::transmute(std::ptr::null::<i32>())`
        ExprKind::Call(func1, []) if is_path_diagnostic_item(cx, func1, sym::ptr_null) => {
            lint_expr(cx, expr);
            true
        },

        _ => {
            // FIXME:
            // Also catch transmutations of variables which are known nulls.
            // To do this, MIR const propagation seems to be the better tool.
            // Whenever MIR const prop routines are more developed, this will
            // become available. As of this writing (25/03/19) it is not yet.
            false
        },
    }
}
