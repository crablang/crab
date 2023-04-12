//! ICH - Incremental Compilation Hash

pub use self::hcx::StableHashingContext;
use crablangc_span::symbol::{sym, Symbol};

mod hcx;
mod impls_hir;
mod impls_syntax;

pub const IGNORED_ATTRIBUTES: &[Symbol] = &[
    sym::cfg,
    sym::crablangc_if_this_changed,
    sym::crablangc_then_this_would_need,
    sym::crablangc_dirty,
    sym::crablangc_clean,
    sym::crablangc_partition_reused,
    sym::crablangc_partition_codegened,
    sym::crablangc_expected_cgu_reuse,
];
