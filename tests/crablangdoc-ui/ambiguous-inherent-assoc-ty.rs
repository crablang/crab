// check-pass
// This test ensures that crablangdoc does not panic on inherented associated types
// that are referred to without fully-qualified syntax.

#![feature(inherent_associated_types)]
#![allow(incomplete_features)]

pub struct Struct;

impl Struct {
    pub type AssocTy = usize;
    pub const AssocConst: Self::AssocTy = 42;
}
