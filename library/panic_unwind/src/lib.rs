//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in CrabLang using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/crablang/crablang/issues/")]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(abi_thiscall)]
#![feature(crablangc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![feature(c_unwind)]
// `real_imp` is unused with Miri, so silence warnings.
#![cfg_attr(miri, allow(dead_code))]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::BoxMeUp;

cfg_if::cfg_if! {
    if #[cfg(target_os = "emscripten")] {
        #[path = "emcc.rs"]
        mod real_imp;
    } else if #[cfg(target_os = "hermit")] {
        #[path = "hermit.rs"]
        mod real_imp;
    } else if #[cfg(target_os = "l4re")] {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod real_imp;
    } else if #[cfg(all(target_env = "msvc", not(target_arch = "arm")))] {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod real_imp;
    } else if #[cfg(any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "solid_asp3",
        all(target_family = "unix", not(target_os = "espidf")),
        all(target_vendor = "fortanix", target_env = "sgx"),
    ))] {
        #[path = "gcc.rs"]
        mod real_imp;
    } else {
        // Targets that don't support unwinding.
        // - family=wasm
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod real_imp;
    }
}

cfg_if::cfg_if! {
    if #[cfg(miri)] {
        // Use the Miri runtime.
        // We still need to also load the normal runtime above, as crablangc expects certain lang
        // items from there to be defined.
        #[path = "miri.rs"]
        mod imp;
    } else {
        // Use the real runtime.
        use real_imp as imp;
    }
}

extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    fn __crablang_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    fn __crablang_foreign_exception() -> !;
}

#[crablangc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __crablang_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    Box::into_raw(imp::cleanup(payload))
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[crablangc_std_internal_symbol]
pub unsafe fn __crablang_start_panic(payload: &mut dyn BoxMeUp) -> u32 {
    let payload = Box::from_raw(payload.take_box());

    imp::panic(payload)
}
