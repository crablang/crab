// compile-flags:-C panic=unwind
// no-prefer-dynamic

#![feature(panic_runtime)]
#![crate_type = "rlib"]

#![no_std]
#![panic_runtime]

#[no_mangle]
pub extern "C" fn __crablang_maybe_catch_panic() {}

#[no_mangle]
pub extern "C" fn __crablang_start_panic() {}

#[no_mangle]
pub extern "C" fn crablang_eh_personality() {}
