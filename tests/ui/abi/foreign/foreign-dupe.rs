// run-pass
// aux-build:foreign_lib.rs
// ignore-wasm32-bare no libc to test ffi with

// Check that we can still call duplicated extern (imported) functions
// which were declared in another crate. See issues #32740 and #32783.


extern crate foreign_lib;

pub fn main() {
    unsafe {
        let x = foreign_lib::crablangrt::crablang_get_test_int();
        assert_eq!(x, foreign_lib::crablangrt2::crablang_get_test_int());
        assert_eq!(x as *const _, foreign_lib::crablangrt3::crablang_get_test_int());
    }
}
