// run-pass
#![allow(improper_ctypes)]

// ignore-wasm32-bare no libc to test ffi with

pub struct TwoU32s {
    one: u32,
    two: u32,
}

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_return_TwoU32s() -> TwoU32s;
}

pub fn main() {
    unsafe {
        let y = crablang_dbg_extern_return_TwoU32s();
        assert_eq!(y.one, 10);
        assert_eq!(y.two, 20);
    }
}
