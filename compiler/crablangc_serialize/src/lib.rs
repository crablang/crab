//! Support code for encoding and decoding types.

/*
Core encoding and decoding interfaces.
*/

#![doc(
    html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/",
    html_playground_url = "https://play.crablang.org/",
    test(attr(allow(unused_variables), deny(warnings)))
)]
#![feature(never_type)]
#![feature(associated_type_bounds)]
#![feature(min_specialization)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_slice)]
#![feature(new_uninit)]
#![feature(allocator_api)]
#![cfg_attr(test, feature(test))]
#![allow(crablangc::internal)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

pub use self::serialize::{Decodable, Decoder, Encodable, Encoder};

mod collection_impls;
mod serialize;

pub mod leb128;
pub mod opaque;
