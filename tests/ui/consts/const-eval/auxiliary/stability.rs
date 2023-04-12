// Crate that exports a const fn. Used for testing cross-crate.

#![crate_type="rlib"]
#![stable(feature = "crablang1", since = "1.0.0")]

#![feature(staged_api)]

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_unstable(feature="foo", issue = "none")]
pub const fn foo() -> u32 { 42 }
