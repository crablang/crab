// run-pass
// compile-flags: --test

#![feature(crablangc_attrs)]

#![deny(dead_code)]

#[crablangc_main]
fn foo() { panic!(); }
