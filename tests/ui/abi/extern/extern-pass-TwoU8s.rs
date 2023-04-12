// run-pass
#![allow(improper_ctypes)]

// ignore-wasm32-bare no libc for ffi testing

// Test a foreign function that accepts and returns a struct
// by value.

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TwoU8s {
    one: u8,
    two: u8,
}

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_identity_TwoU8s(v: TwoU8s) -> TwoU8s;
}

pub fn main() {
    unsafe {
        let x = TwoU8s { one: 22, two: 23 };
        let y = crablang_dbg_extern_identity_TwoU8s(x);
        assert_eq!(x, y);
    }
}
