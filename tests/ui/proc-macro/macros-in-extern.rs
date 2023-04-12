// run-pass
// aux-build:test-macros.rs
// ignore-wasm32

#[macro_use]
extern crate test_macros;

fn main() {
    assert_eq!(unsafe { crablang_get_test_int() }, 1);
    assert_eq!(unsafe { crablang_dbg_extern_identity_u32(0xDEADBEEF) }, 0xDEADBEEF);
}

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    #[empty_attr]
    fn some_definitely_unknown_symbol_which_should_be_removed();

    #[identity_attr]
    fn crablang_get_test_int() -> isize;

    identity!(
        fn crablang_dbg_extern_identity_u32(arg: u32) -> u32;
    );
}
