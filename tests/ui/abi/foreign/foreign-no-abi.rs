// run-pass
// ABI is cdecl by default

// ignore-wasm32-bare no libc to test ffi with
// pretty-expanded FIXME #23616

#![feature(crablangc_private)]

mod crablangrt {
    extern crate libc;

    #[link(name = "crablang_test_helpers", kind = "static")]
    extern "C" {
        pub fn crablang_get_test_int() -> libc::intptr_t;
    }
}

pub fn main() {
    unsafe {
        crablangrt::crablang_get_test_int();
    }
}
