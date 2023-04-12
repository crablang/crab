use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::ty::match_type;
use clippy_utils::{is_lint_allowed, method_calls, paths};
use if_chain::if_chain;
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::{LateContext, LateLintPass};
use crablangc_session::{declare_lint_pass, declare_tool_lint};
use crablangc_span::symbol::Symbol;

declare_clippy_lint! {
    /// ### What it does
    /// Checks for calls to `cx.outer().expn_data()` and suggests to use
    /// the `cx.outer_expn_data()`
    ///
    /// ### Why is this bad?
    /// `cx.outer_expn_data()` is faster and more concise.
    ///
    /// ### Example
    /// ```crablang,ignore
    /// expr.span.ctxt().outer().expn_data()
    /// ```
    ///
    /// Use instead:
    /// ```crablang,ignore
    /// expr.span.ctxt().outer_expn_data()
    /// ```
    pub OUTER_EXPN_EXPN_DATA,
    internal,
    "using `cx.outer_expn().expn_data()` instead of `cx.outer_expn_data()`"
}

declare_lint_pass!(OuterExpnDataPass => [OUTER_EXPN_EXPN_DATA]);

impl<'tcx> LateLintPass<'tcx> for OuterExpnDataPass {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if is_lint_allowed(cx, OUTER_EXPN_EXPN_DATA, expr.hir_id) {
            return;
        }

        let (method_names, arg_lists, spans) = method_calls(expr, 2);
        let method_names: Vec<&str> = method_names.iter().map(Symbol::as_str).collect();
        if_chain! {
            if let ["expn_data", "outer_expn"] = method_names.as_slice();
            let (self_arg, args) = arg_lists[1];
            if args.is_empty();
            let self_ty = cx.typeck_results().expr_ty(self_arg).peel_refs();
            if match_type(cx, self_ty, &paths::SYNTAX_CONTEXT);
            then {
                span_lint_and_sugg(
                    cx,
                    OUTER_EXPN_EXPN_DATA,
                    spans[1].with_hi(expr.span.hi()),
                    "usage of `outer_expn().expn_data()`",
                    "try",
                    "outer_expn_data()".to_string(),
                    Applicability::MachineApplicable,
                );
            }
        }
    }
}
