//! This crates defines the type inference engine.
//!
//! - **Type inference.** The type inference code can be found in the `infer` module;
//!   this code handles low-level equality and subtyping operations. The
//!   type check pass in the compiler is found in the `crablangc_hir_analysis` crate.
//!
//! For more information about how crablangc works, see the [crablangc dev guide].
//!
//! [crablangc dev guide]: https://crablangc-dev-guide.crablang.org/
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(associated_type_bounds)]
#![feature(box_patterns)]
#![feature(control_flow_enum)]
#![feature(extend_one)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(min_specialization)]
#![feature(never_type)]
#![feature(try_blocks)]
#![recursion_limit = "512"] // For crablangdoc

#[macro_use]
extern crate crablangc_macros;
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[macro_use]
extern crate crablangc_data_structures;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate crablangc_middle;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

mod errors;
pub mod infer;
pub mod traits;

fluent_messages! { "../messages.ftl" }
