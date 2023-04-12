use super::WRONG_TRANSMUTE;
use clippy_utils::diagnostics::span_lint;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_middle::ty::{self, Ty};

/// Checks for `wrong_transmute` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, e: &'tcx Expr<'_>, from_ty: Ty<'tcx>, to_ty: Ty<'tcx>) -> bool {
    match (&from_ty.kind(), &to_ty.kind()) {
        (ty::Float(_) | ty::Char, ty::Ref(..) | ty::RawPtr(_)) => {
            span_lint(
                cx,
                WRONG_TRANSMUTE,
                e.span,
                &format!("transmute from a `{from_ty}` to a pointer"),
            );
            true
        },
        _ => false,
    }
}
