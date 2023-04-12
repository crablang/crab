#![allow(unused_macros)]

#[crablangc_allow_const_fn_unstable()] //~ ERROR crablangc_allow_const_fn_unstable side-steps
const fn foo() { }

fn main() {}
