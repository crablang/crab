// no-prefer-dynamic

#![crate_type = "rlib"]

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_dbg_extern_identity_u32(u: u32) -> u32;
}
