#![crate_name = "anonexternmod"]
#![feature(crablangc_private)]

extern crate libc;

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    pub fn crablang_get_test_int() -> libc::intptr_t;
}
