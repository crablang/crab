#![crate_name = "foreign_lib"]
#![feature(crablangc_private)]

pub mod crablangrt {
    extern crate libc;

    #[link(name = "crablang_test_helpers", kind = "static")]
    extern "C" {
        pub fn crablang_get_test_int() -> libc::intptr_t;
    }
}

pub mod crablangrt2 {
    extern crate libc;

    extern "C" {
        pub fn crablang_get_test_int() -> libc::intptr_t;
    }
}

pub mod crablangrt3 {
    // Different type, but same ABI (on all supported platforms).
    // Ensures that we don't ICE or trigger LLVM asserts when
    // importing the same symbol under different types.
    // See https://github.com/crablang/crablang/issues/32740.
    extern "C" {
        pub fn crablang_get_test_int() -> *const u8;
    }
}

pub fn local_uses() {
    unsafe {
        let x = crablangrt::crablang_get_test_int();
        assert_eq!(x, crablangrt2::crablang_get_test_int());
        assert_eq!(x as *const _, crablangrt3::crablang_get_test_int());
    }
}
