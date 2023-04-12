// aux-build:test-macros.rs

#![feature(crablangc_attrs)]
#![warn(unused_extern_crates)]

extern crate test_macros;
//~^ WARN unused extern crate

#[crablangc_error]
fn main() {} //~ ERROR fatal error triggered by #[crablangc_error]
