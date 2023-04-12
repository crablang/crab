// run-pass
// pretty-expanded FIXME #23616
// ignore-wasm32-bare no libc to test ffi with

#![feature(crablangc_private)]

extern crate libc;

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    fn crablang_get_test_int() -> libc::intptr_t;
}

pub fn main() {
    unsafe {
        let _ = crablang_get_test_int();
    }
}
