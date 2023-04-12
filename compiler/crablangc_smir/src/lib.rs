//! The WIP stable interface to crablangc internals.
//!
//! For more information see https://github.com/crablang/project-stable-mir
//!
//! # Note
//!
//! This API is still completely unstable and subject to change.

#![doc(
    html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/",
    test(attr(allow(unused_variables), deny(warnings)))
)]
#![cfg_attr(not(feature = "default"), feature(crablangc_private))]

pub mod crablangc_internal;
pub mod stable_mir;

// Make this module private for now since external users should not call these directly.
mod crablangc_smir;
