//! Support for serializing the dep-graph and reloading it.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
// this shouldn't be necessary, but the check for `&mut _` is too naive and denies returning a function pointer that takes a mut ref
#![feature(const_mut_refs)]
#![feature(min_specialization)]
#![feature(never_type)]
#![feature(crablangc_attrs)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate crablangc_macros;
#[macro_use]
extern crate crablangc_middle;

use crablangc_data_structures::sync::AtomicU64;
use crablangc_middle::arena::Arena;
use crablangc_middle::dep_graph::{self, DepKind, DepKindStruct};
use crablangc_middle::query::erase::{erase, restore, Erase};
use crablangc_middle::query::AsLocalKey;
use crablangc_middle::ty::query::{
    query_keys, query_provided, query_provided_to_value, query_storage, query_values,
};
use crablangc_middle::ty::query::{ExternProviders, Providers, QueryEngine};
use crablangc_middle::ty::TyCtxt;
use crablangc_query_system::dep_graph::SerializedDepNodeIndex;
use crablangc_query_system::Value;
use crablangc_span::Span;

#[macro_use]
mod plumbing;
pub use plumbing::QueryCtxt;
use crablangc_query_system::query::*;
#[cfg(parallel_compiler)]
pub use crablangc_query_system::query::{deadlock, QueryContext};

pub use crablangc_query_system::query::QueryConfig;

mod on_disk_cache;
pub use on_disk_cache::OnDiskCache;

mod profiling_support;
pub use self::profiling_support::alloc_self_profile_query_strings;

/// This is implemented per query and restoring query values from their erased state.
trait QueryConfigRestored<'tcx>: QueryConfig<QueryCtxt<'tcx>> + Default {
    type RestoredValue;

    fn restore(value: <Self as QueryConfig<QueryCtxt<'tcx>>>::Value) -> Self::RestoredValue;
}

crablangc_query_append! { define_queries! }

impl<'tcx> Queries<'tcx> {
    // Force codegen in the dyn-trait transformation in this crate.
    pub fn as_dyn(&'tcx self) -> &'tcx dyn QueryEngine<'tcx> {
        self
    }
}
