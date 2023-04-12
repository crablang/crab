//! checks for `#[inline]` on trait methods without bodies

use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::sugg::DiagnosticExt;
use crablangc_ast::ast::Attribute;
use crablangc_errors::Applicability;
use crablangc_hir::{TraitFn, TraitItem, TraitItemKind};
use crablangc_lint::{LateContext, LateLintPass};
use crablangc_session::{declare_lint_pass, declare_tool_lint};
use crablangc_span::{sym, Symbol};

declare_clippy_lint! {
    /// ### What it does
    /// Checks for `#[inline]` on trait methods without bodies
    ///
    /// ### Why is this bad?
    /// Only implementations of trait methods may be inlined.
    /// The inline attribute is ignored for trait methods without bodies.
    ///
    /// ### Example
    /// ```crablang
    /// trait Animal {
    ///     #[inline]
    ///     fn name(&self) -> &'static str;
    /// }
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub INLINE_FN_WITHOUT_BODY,
    correctness,
    "use of `#[inline]` on trait methods without bodies"
}

declare_lint_pass!(InlineFnWithoutBody => [INLINE_FN_WITHOUT_BODY]);

impl<'tcx> LateLintPass<'tcx> for InlineFnWithoutBody {
    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx TraitItem<'_>) {
        if let TraitItemKind::Fn(_, TraitFn::Required(_)) = item.kind {
            let attrs = cx.tcx.hir().attrs(item.hir_id());
            check_attrs(cx, item.ident.name, attrs);
        }
    }
}

fn check_attrs(cx: &LateContext<'_>, name: Symbol, attrs: &[Attribute]) {
    for attr in attrs {
        if !attr.has_name(sym::inline) {
            continue;
        }

        span_lint_and_then(
            cx,
            INLINE_FN_WITHOUT_BODY,
            attr.span,
            &format!("use of `#[inline]` on trait method `{name}` which has no body"),
            |diag| {
                diag.suggest_remove_item(cx, attr.span, "remove", Applicability::MachineApplicable);
            },
        );
    }
}
