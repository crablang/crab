use super::ERR_EXPECT;
use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::ty::has_debug_impl;
use clippy_utils::ty::is_type_diagnostic_item;
use crablangc_errors::Applicability;
use crablangc_lint::LateContext;
use crablangc_middle::ty;
use crablangc_middle::ty::Ty;
use crablangc_span::{sym, Span};

pub(super) fn check(
    cx: &LateContext<'_>,
    _expr: &crablangc_hir::Expr<'_>,
    recv: &crablangc_hir::Expr<'_>,
    expect_span: Span,
    err_span: Span,
    msrv: &Msrv,
) {
    if_chain! {
        if is_type_diagnostic_item(cx, cx.typeck_results().expr_ty(recv), sym::Result);
        // Test the version to make sure the lint can be showed (expect_err has been
        // introduced in crablang 1.17.0 : https://github.com/crablang/crablang/pull/38982)
        if msrv.meets(msrvs::EXPECT_ERR);

        // Grabs the `Result<T, E>` type
        let result_type = cx.typeck_results().expr_ty(recv);
        // Tests if the T type in a `Result<T, E>` is not None
        if let Some(data_type) = get_data_type(cx, result_type);
        // Tests if the T type in a `Result<T, E>` implements debug
        if has_debug_impl(cx, data_type);

        then {
            span_lint_and_sugg(
                cx,
                ERR_EXPECT,
                err_span.to(expect_span),
                "called `.err().expect()` on a `Result` value",
                "try",
                "expect_err".to_string(),
                Applicability::MachineApplicable
        );
        }
    };
}

/// Given a `Result<T, E>` type, return its data (`T`).
fn get_data_type<'a>(cx: &LateContext<'_>, ty: Ty<'a>) -> Option<Ty<'a>> {
    match ty.kind() {
        ty::Adt(_, substs) if is_type_diagnostic_item(cx, ty, sym::Result) => substs.types().next(),
        _ => None,
    }
}
