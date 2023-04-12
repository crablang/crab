// run-pass
#![allow(improper_ctypes)]

// ignore-wasm32-bare no libc to test ffi with

pub struct TwoU16s {
    one: u16,
    two: u16,
}

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_return_TwoU16s() -> TwoU16s;
}

pub fn main() {
    unsafe {
        let y = crablang_dbg_extern_return_TwoU16s();
        assert_eq!(y.one, 10);
        assert_eq!(y.two, 20);
    }
}
