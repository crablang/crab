//! This crate defines the trait resolution method.
//!
//! - **Traits.** Trait resolution is implemented in the `traits` module.
//!
//! For more information about how crablangc works, see the [crablangc-dev-guide].
//!
//! [crablangc-dev-guide]: https://crablangc-dev-guide.crablang.org/
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(associated_type_bounds)]
#![feature(box_patterns)]
#![feature(control_flow_enum)]
#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(never_type)]
#![feature(result_option_inspect)]
#![feature(type_alias_impl_trait)]
#![feature(min_specialization)]
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
#[macro_use]
extern crate smallvec;

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

pub mod errors;
pub mod infer;
pub mod solve;
pub mod traits;

fluent_messages! { "../messages.ftl" }
