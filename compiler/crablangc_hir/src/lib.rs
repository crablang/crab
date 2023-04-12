//! HIR datatypes. See the [crablangc dev guide] for more info.
//!
//! [crablangc dev guide]: https://crablangc-dev-guide.crablang.org/hir.html

#![feature(associated_type_defaults)]
#![feature(closure_track_caller)]
#![feature(const_btree_len)]
#![feature(let_chains)]
#![feature(min_specialization)]
#![feature(never_type)]
#![feature(crablangc_attrs)]
#![feature(variant_count)]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate crablangc_macros;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate crablangc_data_structures;

extern crate self as crablangc_hir;

mod arena;
pub mod def;
pub mod def_path_hash_map;
pub mod definitions;
pub mod diagnostic_items;
pub mod errors;
pub use crablangc_span::def_id;
mod hir;
pub mod hir_id;
pub mod intravisit;
pub mod lang_items;
pub mod pat_util;
mod stable_hash_impls;
mod target;
pub mod weak_lang_items;

#[cfg(test)]
mod tests;

pub use hir::*;
pub use hir_id::*;
pub use lang_items::{LangItem, LanguageItems};
pub use stable_hash_impls::HashStableContext;
pub use target::{MethodKind, Target};

arena_types!(crablangc_arena::declare_arena);
