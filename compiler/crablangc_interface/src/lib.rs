#![feature(box_patterns)]
#![feature(decl_macro)]
#![feature(internal_output_capture)]
#![feature(thread_spawn_unchecked)]
#![feature(lazy_cell)]
#![feature(try_blocks)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

mod callbacks;
mod errors;
pub mod interface;
mod passes;
mod proc_macro_decls;
mod queries;
pub mod util;

pub use callbacks::setup_callbacks;
pub use interface::{run_compiler, Config};
pub use passes::{DEFAULT_EXTERN_QUERY_PROVIDERS, DEFAULT_QUERY_PROVIDERS};
pub use queries::Queries;

#[cfg(test)]
mod tests;

fluent_messages! { "../messages.ftl" }
