// run-pass
// ignore-wasm32-bare no libc for ffi testing

// Test a call to a function that takes/returns a u64.

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_identity_u64(v: u64) -> u64;
}

pub fn main() {
    unsafe {
        assert_eq!(22, crablang_dbg_extern_identity_u64(22));
    }
}
