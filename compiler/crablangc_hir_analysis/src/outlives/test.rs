use crablangc_errors::struct_span_err;
use crablangc_middle::ty::TyCtxt;
use crablangc_span::symbol::sym;

pub fn test_inferred_outlives(tcx: TyCtxt<'_>) {
    for id in tcx.hir().items() {
        // For unit testing: check for a special "crablangc_outlives"
        // attribute and report an error with various results if found.
        if tcx.has_attr(id.owner_id, sym::crablangc_outlives) {
            let inferred_outlives_of = tcx.inferred_outlives_of(id.owner_id);
            struct_span_err!(
                tcx.sess,
                tcx.def_span(id.owner_id),
                E0640,
                "{:?}",
                inferred_outlives_of
            )
            .emit();
        }
    }
}
