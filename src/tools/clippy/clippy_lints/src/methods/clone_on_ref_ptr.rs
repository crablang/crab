use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::paths;
use clippy_utils::source::snippet_with_context;
use clippy_utils::ty::{is_type_diagnostic_item, match_type};
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_middle::ty;
use crablangc_span::symbol::{sym, Symbol};

use super::CLONE_ON_REF_PTR;

pub(super) fn check(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    method_name: Symbol,
    receiver: &hir::Expr<'_>,
    args: &[hir::Expr<'_>],
) {
    if !(args.is_empty() && method_name == sym::clone) {
        return;
    }
    let obj_ty = cx.typeck_results().expr_ty(receiver).peel_refs();

    if let ty::Adt(_, subst) = obj_ty.kind() {
        let caller_type = if is_type_diagnostic_item(cx, obj_ty, sym::Rc) {
            "Rc"
        } else if is_type_diagnostic_item(cx, obj_ty, sym::Arc) {
            "Arc"
        } else if match_type(cx, obj_ty, &paths::WEAK_RC) || match_type(cx, obj_ty, &paths::WEAK_ARC) {
            "Weak"
        } else {
            return;
        };

        // Sometimes unnecessary ::<_> after Rc/Arc/Weak
        let mut app = Applicability::Unspecified;
        let snippet = snippet_with_context(cx, receiver.span, expr.span.ctxt(), "..", &mut app).0;

        span_lint_and_sugg(
            cx,
            CLONE_ON_REF_PTR,
            expr.span,
            "using `.clone()` on a ref-counted pointer",
            "try this",
            format!("{caller_type}::<{}>::clone(&{snippet})", subst.type_at(0)),
            app,
        );
    }
}
