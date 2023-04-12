use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::{is_expr_identity_function, is_trait_method};
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_span::{source_map::Span, sym};

use super::FILTER_MAP_IDENTITY;

pub(super) fn check(cx: &LateContext<'_>, expr: &hir::Expr<'_>, filter_map_arg: &hir::Expr<'_>, filter_map_span: Span) {
    if is_trait_method(cx, expr, sym::Iterator) && is_expr_identity_function(cx, filter_map_arg) {
        span_lint_and_sugg(
            cx,
            FILTER_MAP_IDENTITY,
            filter_map_span.with_hi(expr.span.hi()),
            "use of `filter_map` with an identity function",
            "try",
            "flatten()".to_string(),
            Applicability::MachineApplicable,
        );
    }
}
