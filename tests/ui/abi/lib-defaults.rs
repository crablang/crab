// run-pass
// dont-check-compiler-stderr (crablang/crablang#54222)

// ignore-wasm32-bare no libc to test ffi with

// compile-flags: -lcrablang_test_helpers

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_identity_u32(x: u32) -> u32;
}

fn main() {
    unsafe {
        crablang_dbg_extern_identity_u32(42);
    }
}
