//! The `crablangc_ast_passes` crate contains passes which validate the AST in `syntax`
//! parsed by `crablangc_parse` and then lowered, after the passes in this crate,
//! by `crablangc_ast_lowering`.
//!
//! The crate also contains other misc AST visitors, e.g. `node_count` and `show_span`.

#![feature(box_patterns)]
#![feature(if_let_guard)]
#![feature(iter_is_partitioned)]
#![feature(let_chains)]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

pub mod ast_validation;
mod errors;
pub mod feature_gate;
pub mod node_count;
pub mod show_span;

fluent_messages! { "../messages.ftl" }
