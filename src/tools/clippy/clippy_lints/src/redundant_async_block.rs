use clippy_utils::{diagnostics::span_lint_and_sugg, source::snippet};
use crablangc_ast::ast::{Expr, ExprKind, Stmt, StmtKind};
use crablangc_ast::visit::Visitor as AstVisitor;
use crablangc_errors::Applicability;
use crablangc_lint::{EarlyContext, EarlyLintPass};
use crablangc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// ### What it does
    /// Checks for `async` block that only returns `await` on a future.
    ///
    /// ### Why is this bad?
    /// It is simpler and more efficient to use the future directly.
    ///
    /// ### Example
    /// ```crablang
    /// async fn f() -> i32 {
    ///     1 + 2
    /// }
    ///
    /// let fut = async {
    ///     f().await
    /// };
    /// ```
    /// Use instead:
    /// ```crablang
    /// async fn f() -> i32 {
    ///     1 + 2
    /// }
    ///
    /// let fut = f();
    /// ```
    #[clippy::version = "1.69.0"]
    pub REDUNDANT_ASYNC_BLOCK,
    nursery,
    "`async { future.await }` can be replaced by `future`"
}
declare_lint_pass!(RedundantAsyncBlock => [REDUNDANT_ASYNC_BLOCK]);

impl EarlyLintPass for RedundantAsyncBlock {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if expr.span.from_expansion() {
            return;
        }
        if let ExprKind::Async(_, block) = &expr.kind && block.stmts.len() == 1 &&
            let Some(Stmt { kind: StmtKind::Expr(last), .. }) = block.stmts.last() &&
            let ExprKind::Await(future) = &last.kind &&
            !future.span.from_expansion() &&
            !await_in_expr(future)
        {
            if captures_value(last) {
                // If the async block captures variables then there is no equivalence.
                return;
            }

            span_lint_and_sugg(
                cx,
                REDUNDANT_ASYNC_BLOCK,
                expr.span,
                "this async expression only awaits a single future",
                "you can reduce it to",
                snippet(cx, future.span, "..").into_owned(),
                Applicability::MachineApplicable,
            );
        }
    }
}

/// Check whether an expression contains `.await`
fn await_in_expr(expr: &Expr) -> bool {
    let mut detector = AwaitDetector::default();
    detector.visit_expr(expr);
    detector.await_found
}

#[derive(Default)]
struct AwaitDetector {
    await_found: bool,
}

impl<'ast> AstVisitor<'ast> for AwaitDetector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        match (&ex.kind, self.await_found) {
            (ExprKind::Await(_), _) => self.await_found = true,
            (_, false) => crablangc_ast::visit::walk_expr(self, ex),
            _ => (),
        }
    }
}

/// Check whether an expression may have captured a local variable.
/// This is done by looking for paths with only one segment, except as
/// a prefix of `.await` since this would be captured by value.
///
/// This function will sometimes return `true` even tough there are no
/// captures happening: at the AST level, it is impossible to
/// dinstinguish a function call from a call to a closure which comes
/// from the local environment.
fn captures_value(expr: &Expr) -> bool {
    let mut detector = CaptureDetector::default();
    detector.visit_expr(expr);
    detector.capture_found
}

#[derive(Default)]
struct CaptureDetector {
    capture_found: bool,
}

impl<'ast> AstVisitor<'ast> for CaptureDetector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        match (&ex.kind, self.capture_found) {
            (ExprKind::Await(fut), _) if matches!(fut.kind, ExprKind::Path(..)) => (),
            (ExprKind::Path(_, path), _) if path.segments.len() == 1 => self.capture_found = true,
            (_, false) => crablangc_ast::visit::walk_expr(self, ex),
            _ => (),
        }
    }
}
