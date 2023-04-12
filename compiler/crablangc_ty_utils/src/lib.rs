//! Various checks
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(box_patterns)]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate crablangc_middle;
#[macro_use]
extern crate tracing;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;
use crablangc_middle::ty::query::Providers;

mod abi;
mod assoc;
mod common_traits;
mod consts;
mod errors;
mod implied_bounds;
pub mod instance;
mod layout;
mod layout_sanity_check;
mod needs_drop;
pub mod representability;
mod structural_match;
mod ty;

fluent_messages! { "../messages.ftl" }

pub fn provide(providers: &mut Providers) {
    abi::provide(providers);
    assoc::provide(providers);
    common_traits::provide(providers);
    consts::provide(providers);
    implied_bounds::provide(providers);
    layout::provide(providers);
    needs_drop::provide(providers);
    representability::provide(providers);
    ty::provide(providers);
    instance::provide(providers);
    structural_match::provide(providers);
}
