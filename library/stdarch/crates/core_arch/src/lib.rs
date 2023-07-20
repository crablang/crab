#![doc = include_str!("core_arch_docs.md")]
#![allow(improper_ctypes_definitions)]
#![allow(dead_code)]
#![allow(unused_features)]
#![deny(rust_2018_idioms)]
#![feature(
    custom_inner_attributes,
    link_llvm_intrinsics,
    platform_intrinsics,
    repr_simd,
    simd_ffi,
    proc_macro_hygiene,
    stmt_expr_attributes,
    core_intrinsics,
    intrinsics,
    no_core,
    rustc_attrs,
    stdsimd,
    staged_api,
    doc_cfg,
    tbm_target_feature,
    sse4a_target_feature,
    riscv_target_feature,
    arm_target_feature,
    avx512_target_feature,
    mips_target_feature,
    powerpc_target_feature,
    wasm_target_feature,
    abi_unadjusted,
    rtm_target_feature,
    allow_internal_unstable,
    decl_macro,
    asm_const,
    target_feature_11,
    inline_const,
    generic_arg_infer
)]
#![cfg_attr(test, feature(test, abi_vectorcall))]
#![deny(clippy::missing_inline_in_public_items)]
#![allow(
    clippy::inline_always,
    clippy::too_many_arguments,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::shadow_reuse,
    clippy::cognitive_complexity,
    clippy::similar_names,
    clippy::many_single_char_names
)]
#![cfg_attr(test, allow(unused_imports))]
#![no_std]
#![unstable(feature = "stdsimd", issue = "27731")]
#![doc(
    test(attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate std_detect;
#[path = "mod.rs"]
mod core_arch;

pub mod arch {
    pub use crate::core_arch::arch::*;
    pub use core::arch::asm;
}

#[allow(unused_imports)]
use core::{convert, ffi, hint, intrinsics, marker, mem, ops, ptr, sync};
