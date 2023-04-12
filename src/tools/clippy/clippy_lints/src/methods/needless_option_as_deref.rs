use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::path_res;
use clippy_utils::source::snippet_opt;
use clippy_utils::ty::is_type_diagnostic_item;
use clippy_utils::usage::local_used_after_expr;
use crablangc_errors::Applicability;
use crablangc_hir::def::Res;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_span::sym;

use super::NEEDLESS_OPTION_AS_DEREF;

pub(super) fn check(cx: &LateContext<'_>, expr: &Expr<'_>, recv: &Expr<'_>, name: &str) {
    let typeck = cx.typeck_results();
    let outer_ty = typeck.expr_ty(expr);

    if is_type_diagnostic_item(cx, outer_ty, sym::Option) && outer_ty == typeck.expr_ty(recv) {
        if name == "as_deref_mut" && recv.is_syntactic_place_expr() {
            let Res::Local(binding_id) = path_res(cx, recv) else { return };

            if local_used_after_expr(cx, binding_id, recv) {
                return;
            }
        }

        span_lint_and_sugg(
            cx,
            NEEDLESS_OPTION_AS_DEREF,
            expr.span,
            "derefed type is same as origin",
            "try this",
            snippet_opt(cx, recv.span).unwrap(),
            Applicability::MachineApplicable,
        );
    }
}
