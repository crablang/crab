#![feature(array_windows)]
#![feature(is_sorted)]
#![recursion_limit = "256"]
#![allow(rustc::potential_query_instability)]
#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate rustc_middle;

use rustc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use rustc_fluent_macro::fluent_messages;
use rustc_hir::lang_items::LangItem;
use rustc_middle::query::{Providers, TyCtxtAt};
use rustc_middle::traits;
use rustc_middle::ty::adjustment::CustomCoerceUnsized;
use rustc_middle::ty::{self, Ty};

mod collector;
mod errors;
mod partitioning;
mod polymorphize;
mod util;

fluent_messages! { "../messages.ftl" }

fn custom_coerce_unsize_info<'tcx>(
    tcx: TyCtxtAt<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> CustomCoerceUnsized {
    let trait_ref = ty::TraitRef::from_lang_item(
        tcx.tcx,
        LangItem::CoerceUnsized,
        tcx.span,
        [source_ty, target_ty],
    );

    match tcx.codegen_select_candidate((ty::ParamEnv::reveal_all(), trait_ref)) {
        Ok(traits::ImplSource::UserDefined(traits::ImplSourceUserDefinedData {
            impl_def_id,
            ..
        })) => tcx.coerce_unsized_info(impl_def_id).custom_kind.unwrap(),
        impl_source => {
            bug!("invalid `CoerceUnsized` impl_source: {:?}", impl_source);
        }
    }
}

pub fn provide(providers: &mut Providers) {
    partitioning::provide(providers);
    polymorphize::provide(providers);
}
