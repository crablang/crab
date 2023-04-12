#![feature(array_windows)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate crablangc_middle;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_hir::lang_items::LangItem;
use crablangc_macros::fluent_messages;
use crablangc_middle::traits;
use crablangc_middle::ty::adjustment::CustomCoerceUnsized;
use crablangc_middle::ty::query::{Providers, TyCtxtAt};
use crablangc_middle::ty::{self, Ty};

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
    let trait_ref =
        ty::Binder::dummy(tcx.mk_trait_ref(LangItem::CoerceUnsized, [source_ty, target_ty]));

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
