#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(decl_macro)]
#![feature(drain_filter)]
#![feature(generators)]
#![feature(iter_from_generator)]
#![feature(let_chains)]
#![feature(proc_macro_internals)]
#![feature(macro_metavar_expr)]
#![feature(min_specialization)]
#![feature(slice_as_chunks)]
#![feature(tcrablanged_len)]
#![feature(try_blocks)]
#![feature(never_type)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

extern crate proc_macro;

#[macro_use]
extern crate crablangc_macros;
#[macro_use]
extern crate crablangc_middle;

#[macro_use]
extern crate tracing;

pub use rmeta::{provide, provide_extern};
use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_macros::fluent_messages;

mod dependency_format;
mod foreign_modules;
mod native_libs;
mod rmeta;

pub mod creader;
pub mod errors;
pub mod fs;
pub mod locator;

pub use fs::{emit_wrapper_file, METADATA_FILENAME};
pub use native_libs::find_native_static_library;
pub use rmeta::{encode_metadata, EncodedMetadata, METADATA_HEADER};

fluent_messages! { "../messages.ftl" }
