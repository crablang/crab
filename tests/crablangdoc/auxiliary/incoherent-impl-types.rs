#![feature(crablangc_attrs)]

#[crablangc_has_incoherent_inherent_impls]
pub trait FooTrait {}

#[crablangc_has_incoherent_inherent_impls]
pub struct FooStruct;
