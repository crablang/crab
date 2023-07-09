use super::ObjectSafetyViolation;

use crate::infer::InferCtxt;
use rustc_data_structures::fx::FxIndexSet;
use rustc_errors::{struct_span_err, DiagnosticBuilder, ErrorGuaranteed, MultiSpan};
use rustc_hir as hir;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use std::fmt;
use std::iter;

impl<'tcx> InferCtxt<'tcx> {
    pub fn report_extra_impl_obligation(
        &self,
        error_span: Span,
        impl_item_def_id: LocalDefId,
        trait_item_def_id: DefId,
        requirement: &dyn fmt::Display,
    ) -> DiagnosticBuilder<'tcx, ErrorGuaranteed> {
        let mut err = struct_span_err!(
            self.tcx.sess,
            error_span,
            E0276,
            "impl has stricter requirements than trait"
        );

        if !self.tcx.is_impl_trait_in_trait(trait_item_def_id) {
            if let Some(span) = self.tcx.hir().span_if_local(trait_item_def_id) {
                let item_name = self.tcx.item_name(impl_item_def_id.to_def_id());
                err.span_label(span, format!("definition of `{}` from trait", item_name));
            }
        }

        err.span_label(error_span, format!("impl has extra requirement {}", requirement));

        err
    }
}

pub fn report_object_safety_error<'tcx>(
    tcx: TyCtxt<'tcx>,
    span: Span,
    trait_def_id: DefId,
    violations: &[ObjectSafetyViolation],
) -> DiagnosticBuilder<'tcx, ErrorGuaranteed> {
    let trait_str = tcx.def_path_str(trait_def_id);
    let trait_span = tcx.hir().get_if_local(trait_def_id).and_then(|node| match node {
        hir::Node::Item(item) => Some(item.ident.span),
        _ => None,
    });
    let mut err = struct_span_err!(
        tcx.sess,
        span,
        E0038,
        "the trait `{}` cannot be made into an object",
        trait_str
    );
    err.span_label(span, format!("`{}` cannot be made into an object", trait_str));

    let mut reported_violations = FxIndexSet::default();
    let mut multi_span = vec![];
    let mut messages = vec![];
    for violation in violations {
        if let ObjectSafetyViolation::SizedSelf(sp) = &violation && !sp.is_empty() {
            // Do not report `SizedSelf` without spans pointing at `SizedSelf` obligations
            // with a `Span`.
            reported_violations.insert(ObjectSafetyViolation::SizedSelf(vec![].into()));
        }
        if reported_violations.insert(violation.clone()) {
            let spans = violation.spans();
            let msg = if trait_span.is_none() || spans.is_empty() {
                format!("the trait cannot be made into an object because {}", violation.error_msg())
            } else {
                format!("...because {}", violation.error_msg())
            };
            if spans.is_empty() {
                err.note(msg);
            } else {
                for span in spans {
                    multi_span.push(span);
                    messages.push(msg.clone());
                }
            }
        }
    }
    let has_multi_span = !multi_span.is_empty();
    let mut note_span = MultiSpan::from_spans(multi_span.clone());
    if let (Some(trait_span), true) = (trait_span, has_multi_span) {
        note_span.push_span_label(trait_span, "this trait cannot be made into an object...");
    }
    for (span, msg) in iter::zip(multi_span, messages) {
        note_span.push_span_label(span, msg);
    }
    err.span_note(
        note_span,
        "for a trait to be \"object safe\" it needs to allow building a vtable to allow the call \
         to be resolvable dynamically; for more information visit \
         <https://doc.rust-lang.org/reference/items/traits.html#object-safety>",
    );
    if trait_span.is_some() {
        let mut reported_violations: Vec<_> = reported_violations.into_iter().collect();
        reported_violations.sort();
        for violation in reported_violations {
            // Only provide the help if its a local trait, otherwise it's not actionable.
            violation.solution(&mut err);
        }
    }
    err
}
