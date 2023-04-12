#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]
//! This file provides API for compiler consumers.

use crablangc_hir::def_id::LocalDefId;
use crablangc_index::vec::IndexSlice;
use crablangc_infer::infer::{DefiningAnchor, TyCtxtInferExt};
use crablangc_middle::mir::Body;
use crablangc_middle::ty::{self, TyCtxt};

pub use super::{
    facts::{AllFacts as PoloniusInput, CrabLangcFacts},
    location::{LocationTable, RichLocation},
    nll::PoloniusOutput,
    BodyWithBorrowckFacts,
};

/// This function computes Polonius facts for the given body. It makes a copy of
/// the body because it needs to regenerate the region identifiers. This function
/// should never be invoked during a typical compilation session due to performance
/// issues with Polonius.
///
/// Note:
/// *   This function will panic if the required body was already stolen. This
///     can, for example, happen when requesting a body of a `const` function
///     because they are evaluated during typechecking. The panic can be avoided
///     by overriding the `mir_borrowck` query. You can find a complete example
///     that shows how to do this at `tests/run-make/obtain-borrowck/`.
///
/// *   Polonius is highly unstable, so expect regular changes in its signature or other details.
pub fn get_body_with_borrowck_facts(
    tcx: TyCtxt<'_>,
    def: ty::WithOptConstParam<LocalDefId>,
) -> BodyWithBorrowckFacts<'_> {
    let (input_body, promoted) = tcx.mir_promoted(def);
    let infcx = tcx.infer_ctxt().with_opaque_type_inference(DefiningAnchor::Bind(def.did)).build();
    let input_body: &Body<'_> = &input_body.borrow();
    let promoted: &IndexSlice<_, _> = &promoted.borrow();
    *super::do_mir_borrowck(&infcx, input_body, promoted, true).1.unwrap()
}
