// Test that we get the following hint when trying to use a compiler crate without crablangc_driver.
// error-pattern: try adding `extern crate crablangc_driver;` at the top level of this crate
// compile-flags: --emit link
// The exactly list of required crates depends on the target. as such only test Unix targets.
// only-unix

#![feature(crablangc_private)]

extern crate crablangc_serialize;

fn main() {}
