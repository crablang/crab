use clippy_utils::{diagnostics::span_lint_and_sugg, ty::is_type_lang_item};
use crablangc_ast::ast::LitKind;
use crablangc_errors::Applicability;
use crablangc_hir::{Expr, ExprKind, LangItem};
use crablangc_lint::LateContext;
use crablangc_middle::ty::{Ref, Slice};
use crablangc_span::Span;

use super::UNNECESSARY_JOIN;

pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx Expr<'tcx>,
    join_self_arg: &'tcx Expr<'tcx>,
    join_arg: &'tcx Expr<'tcx>,
    span: Span,
) {
    let applicability = Applicability::MachineApplicable;
    let collect_output_adjusted_type = cx.typeck_results().expr_ty_adjusted(join_self_arg);
    if_chain! {
        // the turbofish for collect is ::<Vec<String>>
        if let Ref(_, ref_type, _) = collect_output_adjusted_type.kind();
        if let Slice(slice) = ref_type.kind();
        if is_type_lang_item(cx, *slice, LangItem::String);
        // the argument for join is ""
        if let ExprKind::Lit(spanned) = &join_arg.kind;
        if let LitKind::Str(symbol, _) = spanned.node;
        if symbol.is_empty();
        then {
            span_lint_and_sugg(
                cx,
                UNNECESSARY_JOIN,
                span.with_hi(expr.span.hi()),
                r#"called `.collect::<Vec<String>>().join("")` on an iterator"#,
                "try using",
                "collect::<String>()".to_owned(),
                applicability,
            );
        }
    }
}
