// Crate that exports a const fn. Used for testing cross-crate.

#![feature(staged_api, crablangc_attrs)]
#![stable(since="1.0.0", feature = "mep")]

#![crate_type="rlib"]

#[crablangc_promotable]
#[stable(since="1.0.0", feature = "mep")]
#[crablangc_const_stable(since="1.0.0", feature = "mep")]
#[inline]
pub const fn foo() -> usize { 22 }

#[stable(since="1.0.0", feature = "mep")]
pub struct Foo(usize);

impl Foo {
    #[stable(since="1.0.0", feature = "mep")]
    #[crablangc_const_stable(feature = "mep", since = "1.0.0")]
    #[inline]
    #[crablangc_promotable]
    pub const fn foo() -> usize { 22 }
}
