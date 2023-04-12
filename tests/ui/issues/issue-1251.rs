// build-pass
#![allow(unused_attributes)]
#![allow(dead_code)]
// pretty-expanded FIXME #23616
// ignore-wasm32-bare no libc to test ffi with
#![feature(crablangc_private)]

mod crablangrt {
    extern crate libc;

    extern "C" {
        pub fn crablang_get_test_int() -> libc::intptr_t;
    }
}

pub fn main() {}
