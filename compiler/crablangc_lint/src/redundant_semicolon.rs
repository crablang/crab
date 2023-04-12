use crate::{lints::RedundantSemicolonsDiag, EarlyContext, EarlyLintPass, LintContext};
use crablangc_ast::{Block, StmtKind};
use crablangc_span::Span;

declare_lint! {
    /// The `redundant_semicolons` lint detects unnecessary trailing
    /// semicolons.
    ///
    /// ### Example
    ///
    /// ```crablang
    /// let _ = 123;;
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Extra semicolons are not needed, and may be removed to avoid confusion
    /// and visual clutter.
    pub REDUNDANT_SEMICOLONS,
    Warn,
    "detects unnecessary trailing semicolons"
}

declare_lint_pass!(RedundantSemicolons => [REDUNDANT_SEMICOLONS]);

impl EarlyLintPass for RedundantSemicolons {
    fn check_block(&mut self, cx: &EarlyContext<'_>, block: &Block) {
        let mut seq = None;
        for stmt in block.stmts.iter() {
            match (&stmt.kind, &mut seq) {
                (StmtKind::Empty, None) => seq = Some((stmt.span, false)),
                (StmtKind::Empty, Some(seq)) => *seq = (seq.0.to(stmt.span), true),
                (_, seq) => maybe_lint_redundant_semis(cx, seq),
            }
        }
        maybe_lint_redundant_semis(cx, &mut seq);
    }
}

fn maybe_lint_redundant_semis(cx: &EarlyContext<'_>, seq: &mut Option<(Span, bool)>) {
    if let Some((span, multiple)) = seq.take() {
        // FIXME: Find a better way of ignoring the trailing
        // semicolon from macro expansion
        if span == crablangc_span::DUMMY_SP {
            return;
        }

        cx.emit_spanned_lint(
            REDUNDANT_SEMICOLONS,
            span,
            RedundantSemicolonsDiag { multiple, suggestion: span },
        );
    }
}
