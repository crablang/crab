use super::TRANSMUTE_INT_TO_BOOL;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::sugg;
use crablangc_ast as ast;
use crablangc_errors::Applicability;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_middle::ty::{self, Ty};
use std::borrow::Cow;

/// Checks for `transmute_int_to_bool` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    e: &'tcx Expr<'_>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    arg: &'tcx Expr<'_>,
) -> bool {
    match (&from_ty.kind(), &to_ty.kind()) {
        (ty::Int(ty::IntTy::I8) | ty::Uint(ty::UintTy::U8), ty::Bool) => {
            span_lint_and_then(
                cx,
                TRANSMUTE_INT_TO_BOOL,
                e.span,
                &format!("transmute from a `{from_ty}` to a `bool`"),
                |diag| {
                    let arg = sugg::Sugg::hir(cx, arg, "..");
                    let zero = sugg::Sugg::NonParen(Cow::from("0"));
                    diag.span_suggestion(
                        e.span,
                        "consider using",
                        sugg::make_binop(ast::BinOpKind::Ne, &arg, &zero).to_string(),
                        Applicability::Unspecified,
                    );
                },
            );
            true
        },
        _ => false,
    }
}
