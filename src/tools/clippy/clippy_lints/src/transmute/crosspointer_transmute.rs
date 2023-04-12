use super::CROSSPOINTER_TRANSMUTE;
use clippy_utils::diagnostics::span_lint;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_middle::ty::{self, Ty};

/// Checks for `crosspointer_transmute` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(cx: &LateContext<'tcx>, e: &'tcx Expr<'_>, from_ty: Ty<'tcx>, to_ty: Ty<'tcx>) -> bool {
    match (&from_ty.kind(), &to_ty.kind()) {
        (ty::RawPtr(from_ptr), _) if from_ptr.ty == to_ty => {
            span_lint(
                cx,
                CROSSPOINTER_TRANSMUTE,
                e.span,
                &format!("transmute from a type (`{from_ty}`) to the type that it points to (`{to_ty}`)"),
            );
            true
        },
        (_, ty::RawPtr(to_ptr)) if to_ptr.ty == from_ty => {
            span_lint(
                cx,
                CROSSPOINTER_TRANSMUTE,
                e.span,
                &format!("transmute from a type (`{from_ty}`) to a pointer to that type (`{to_ty}`)"),
            );
            true
        },
        _ => false,
    }
}
