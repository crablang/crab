use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::get_parent_expr;
use clippy_utils::ty::is_type_diagnostic_item;
use if_chain::if_chain;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_span::source_map::Span;
use crablangc_span::sym;

use super::FILETYPE_IS_FILE;

pub(super) fn check(cx: &LateContext<'_>, expr: &hir::Expr<'_>, recv: &hir::Expr<'_>) {
    let ty = cx.typeck_results().expr_ty(recv);

    if !is_type_diagnostic_item(cx, ty, sym::FileType) {
        return;
    }

    let span: Span;
    let verb: &str;
    let lint_unary: &str;
    let help_unary: &str;
    if_chain! {
        if let Some(parent) = get_parent_expr(cx, expr);
        if let hir::ExprKind::Unary(op, _) = parent.kind;
        if op == hir::UnOp::Not;
        then {
            lint_unary = "!";
            verb = "denies";
            help_unary = "";
            span = parent.span;
        } else {
            lint_unary = "";
            verb = "covers";
            help_unary = "!";
            span = expr.span;
        }
    }
    let lint_msg = format!("`{lint_unary}FileType::is_file()` only {verb} regular files");
    let help_msg = format!("use `{help_unary}FileType::is_dir()` instead");
    span_lint_and_help(cx, FILETYPE_IS_FILE, span, &lint_msg, None, &help_msg);
}
