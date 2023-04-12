#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(min_specialization)]
#![feature(never_type)]
#![feature(lazy_cell)]
#![feature(option_get_or_insert_default)]
#![feature(crablangc_attrs)]
#![feature(map_many_mut)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate crablangc_macros;
pub mod errors;

#[macro_use]
extern crate tracing;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

pub mod cgu_reuse_tracker;
pub mod utils;
pub use lint::{declare_lint, declare_lint_pass, declare_tool_lint, impl_lint_pass};
pub use crablangc_lint_defs as lint;
pub mod parse;

mod code_stats;
#[macro_use]
pub mod config;
pub mod cstore;
pub mod filesearch;
mod options;
pub mod search_paths;

mod session;
pub use session::*;

pub mod output;

pub use getopts;

fluent_messages! { "../messages.ftl" }

/// Requirements for a `StableHashingContext` to be used in this crate.
/// This is a hack to allow using the `HashStable_Generic` derive macro
/// instead of implementing everything in `crablangc_middle`.
pub trait HashStableContext: crablangc_ast::HashStableContext + crablangc_hir::HashStableContext {}
