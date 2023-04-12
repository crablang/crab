#![feature(array_windows)]
#![feature(associated_type_bounds)]
#![feature(associated_type_defaults)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(macro_metavar_expr)]
#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_internals)]
#![feature(proc_macro_span)]
#![feature(crablangc_attrs)]
#![feature(try_blocks)]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]

#[macro_use]
extern crate crablangc_macros;

#[macro_use]
extern crate tracing;

extern crate proc_macro as pm;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

mod placeholders;
mod proc_macro_server;

pub use mbe::macro_rules::compile_declarative_macro;
pub(crate) use crablangc_span::hygiene;
pub mod base;
pub mod build;
#[macro_use]
pub mod config;
pub mod errors;
pub mod expand;
pub mod module;

// FIXME(Nilstrieb) Translate proc_macro diagnostics
#[allow(crablangc::untranslatable_diagnostic)]
pub mod proc_macro;

// FIXME(Nilstrieb) Translate macro_rules diagnostics
#[allow(crablangc::untranslatable_diagnostic)]
pub(crate) mod mbe;

// HACK(Centril, #64197): These shouldn't really be here.
// Rather, they should be with their respective modules which are defined in other crates.
// However, since for now constructing a `ParseSess` sorta requires `config` from this crate,
// these tests will need to live here in the interim.

#[cfg(test)]
mod tests;
#[cfg(test)]
mod parse {
    mod tests;
}
#[cfg(test)]
mod tokenstream {
    mod tests;
}
#[cfg(test)]
mod mut_visit {
    mod tests;
}

fluent_messages! { "../messages.ftl" }
