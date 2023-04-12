use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::{is_expr_identity_function, is_trait_method};
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_span::{source_map::Span, sym};

use super::FLAT_MAP_IDENTITY;

/// lint use of `flat_map` for `Iterators` where `flatten` would be sufficient
pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'_>,
    flat_map_arg: &'tcx hir::Expr<'_>,
    flat_map_span: Span,
) {
    if is_trait_method(cx, expr, sym::Iterator) && is_expr_identity_function(cx, flat_map_arg) {
        span_lint_and_sugg(
            cx,
            FLAT_MAP_IDENTITY,
            flat_map_span.with_hi(expr.span.hi()),
            "use of `flat_map` with an identity function",
            "try",
            "flatten()".to_string(),
            Applicability::MachineApplicable,
        );
    }
}
