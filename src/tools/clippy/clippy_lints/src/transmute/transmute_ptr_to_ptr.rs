use super::TRANSMUTE_PTR_TO_PTR;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::sugg;
use crablangc_errors::Applicability;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_middle::ty::{self, Ty};

/// Checks for `transmute_ptr_to_ptr` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    e: &'tcx Expr<'_>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    arg: &'tcx Expr<'_>,
) -> bool {
    match (&from_ty.kind(), &to_ty.kind()) {
        (ty::RawPtr(_), ty::RawPtr(to_ty)) => {
            span_lint_and_then(
                cx,
                TRANSMUTE_PTR_TO_PTR,
                e.span,
                "transmute from a pointer to a pointer",
                |diag| {
                    if let Some(arg) = sugg::Sugg::hir_opt(cx, arg) {
                        let sugg = arg.as_ty(cx.tcx.mk_ptr(*to_ty));
                        diag.span_suggestion(e.span, "try", sugg, Applicability::Unspecified);
                    }
                },
            );
            true
        },
        _ => false,
    }
}
