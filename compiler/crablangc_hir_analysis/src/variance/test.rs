use crablangc_middle::ty::TyCtxt;
use crablangc_span::symbol::sym;

use crate::errors;

pub fn test_variance(tcx: TyCtxt<'_>) {
    // For unit testing: check for a special "crablangc_variance"
    // attribute and report an error with various results if found.
    for id in tcx.hir().items() {
        if tcx.has_attr(id.owner_id, sym::crablangc_variance) {
            let variances_of = tcx.variances_of(id.owner_id);

            tcx.sess.emit_err(errors::VariancesOf {
                span: tcx.def_span(id.owner_id),
                variances_of: format!("{variances_of:?}"),
            });
        }
    }
}
