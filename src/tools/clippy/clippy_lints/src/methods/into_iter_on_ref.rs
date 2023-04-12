use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::is_trait_method;
use clippy_utils::ty::has_iter_method;
use if_chain::if_chain;
use crablangc_errors::Applicability;
use crablangc_hir as hir;
use crablangc_lint::LateContext;
use crablangc_middle::ty::{self, Ty};
use crablangc_span::source_map::Span;
use crablangc_span::symbol::{sym, Symbol};

use super::INTO_ITER_ON_REF;

pub(super) fn check(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    method_span: Span,
    method_name: Symbol,
    receiver: &hir::Expr<'_>,
) {
    let self_ty = cx.typeck_results().expr_ty_adjusted(receiver);
    if_chain! {
        if let ty::Ref(..) = self_ty.kind();
        if method_name == sym::into_iter;
        if is_trait_method(cx, expr, sym::IntoIterator);
        if let Some((kind, method_name)) = ty_has_iter_method(cx, self_ty);
        then {
            span_lint_and_sugg(
                cx,
                INTO_ITER_ON_REF,
                method_span,
                &format!(
                    "this `.into_iter()` call is equivalent to `.{method_name}()` and will not consume the `{kind}`",
                ),
                "call directly",
                method_name.to_string(),
                Applicability::MachineApplicable,
            );
        }
    }
}

fn ty_has_iter_method(cx: &LateContext<'_>, self_ref_ty: Ty<'_>) -> Option<(Symbol, &'static str)> {
    has_iter_method(cx, self_ref_ty).map(|ty_name| {
        let ty::Ref(_, _, mutbl) = self_ref_ty.kind() else {
            unreachable!()
        };
        let method_name = match mutbl {
            hir::Mutability::Not => "iter",
            hir::Mutability::Mut => "iter_mut",
        };
        (ty_name, method_name)
    })
}
