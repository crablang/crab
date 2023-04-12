use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::{match_def_path, paths, sugg};
use if_chain::if_chain;
use crablangc_ast::util::parser::AssocOp;
use crablangc_errors::Applicability;
use crablangc_hir::def::{DefKind, Res};
use crablangc_hir::{BinOpKind, Expr, ExprKind};
use crablangc_lint::LateContext;
use crablangc_middle::ty;
use crablangc_span::source_map::Spanned;

use super::FLOAT_EQUALITY_WITHOUT_ABS;

pub(crate) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx Expr<'_>,
    op: BinOpKind,
    lhs: &'tcx Expr<'_>,
    rhs: &'tcx Expr<'_>,
) {
    let (lhs, rhs) = match op {
        BinOpKind::Lt => (lhs, rhs),
        BinOpKind::Gt => (rhs, lhs),
        _ => return,
    };

    if_chain! {
        // left hand side is a subtraction
        if let ExprKind::Binary(
            Spanned {
                node: BinOpKind::Sub,
                ..
            },
            val_l,
            val_r,
        ) = lhs.kind;

        // right hand side matches either f32::EPSILON or f64::EPSILON
        if let ExprKind::Path(ref epsilon_path) = rhs.kind;
        if let Res::Def(DefKind::AssocConst, def_id) = cx.qpath_res(epsilon_path, rhs.hir_id);
        if match_def_path(cx, def_id, &paths::F32_EPSILON) || match_def_path(cx, def_id, &paths::F64_EPSILON);

        // values of the subtractions on the left hand side are of the type float
        let t_val_l = cx.typeck_results().expr_ty(val_l);
        let t_val_r = cx.typeck_results().expr_ty(val_r);
        if let ty::Float(_) = t_val_l.kind();
        if let ty::Float(_) = t_val_r.kind();

        then {
            let sug_l = sugg::Sugg::hir(cx, val_l, "..");
            let sug_r = sugg::Sugg::hir(cx, val_r, "..");
            // format the suggestion
            let suggestion = format!("{}.abs()", sugg::make_assoc(AssocOp::Subtract, &sug_l, &sug_r).maybe_par());
            // spans the lint
            span_lint_and_then(
                cx,
                FLOAT_EQUALITY_WITHOUT_ABS,
                expr.span,
                "float equality check without `.abs()`",
                | diag | {
                    diag.span_suggestion(
                        lhs.span,
                        "add `.abs()`",
                        suggestion,
                        Applicability::MaybeIncorrect,
                    );
                }
            );
        }
    }
}
