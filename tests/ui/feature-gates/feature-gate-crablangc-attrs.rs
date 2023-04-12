// Test that `#[crablangc_*]` attributes are gated by `crablangc_attrs` feature gate.

#![feature(decl_macro)]

mod crablangc { pub macro unknown() {} }
mod unknown { pub macro crablangc() {} }

#[crablangc::unknown]
//~^ ERROR attributes starting with `crablangc` are reserved for use by the `crablangc` compiler
//~| ERROR expected attribute, found macro `crablangc::unknown`
fn f() {}

#[unknown::crablangc]
//~^ ERROR attributes starting with `crablangc` are reserved for use by the `crablangc` compiler
//~| ERROR expected attribute, found macro `unknown::crablangc`
fn g() {}

#[crablangc_dummy]
//~^ ERROR the `#[crablangc_dummy]` attribute is just used for crablangc unit tests
#[crablangc_unknown]
//~^ ERROR attributes starting with `crablangc` are reserved for use by the `crablangc` compiler
//~| ERROR cannot find attribute `crablangc_unknown` in this scope
fn main() {}
