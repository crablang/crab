// run-pass
// ignore-wasm32-bare no libc for ffi testing

// Test a function that takes/returns a u8.

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_identity_u8(v: u8) -> u8;
}

pub fn main() {
    unsafe {
        assert_eq!(22, crablang_dbg_extern_identity_u8(22));
    }
}
