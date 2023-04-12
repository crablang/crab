use super::ITER_NEXT_LOOP;
use clippy_utils::diagnostics::span_lint;
use clippy_utils::is_trait_method;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_span::sym;

pub(super) fn check(cx: &LateContext<'_>, arg: &Expr<'_>) {
    if is_trait_method(cx, arg, sym::Iterator) {
        span_lint(
            cx,
            ITER_NEXT_LOOP,
            arg.span,
            "you are iterating over `Iterator::next()` which is an Option; this will compile but is \
            probably not what you want",
        );
    }
}
