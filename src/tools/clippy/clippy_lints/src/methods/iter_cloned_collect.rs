use crate::methods::utils::derefs_to_slice;
use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::ty::is_type_diagnostic_item;
use if_chain::if_chain;
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_span::sym;

use super::ITER_CLONED_COLLECT;

pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, method_name: &str, expr: &hir::Expr<'_>, recv: &'tcx hir::Expr<'_>) {
    if_chain! {
        if is_type_diagnostic_item(cx, cx.typeck_results().expr_ty(expr), sym::Vec);
        if let Some(slice) = derefs_to_slice(cx, recv, cx.typeck_results().expr_ty(recv));
        if let Some(to_replace) = expr.span.trim_start(slice.span.source_callsite());

        then {
            span_lint_and_sugg(
                cx,
                ITER_CLONED_COLLECT,
                to_replace,
                &format!("called `iter().{method_name}().collect()` on a slice to create a `Vec`. Calling `to_vec()` is both faster and \
                more readable"),
                "try",
                ".to_vec()".to_string(),
                Applicability::MachineApplicable,
            );
        }
    }
}
