// check-pass

#![feature(negative_impls)]
#![feature(crablangc_attrs)]
#![feature(trait_alias)]
#![feature(with_negative_coherence)]

trait A {}
trait B {}
trait AB = A + B;

impl !A for u32 {}

#[crablangc_strict_coherence]
trait C {}
impl<T: AB> C for T {}
impl C for u32 {}

fn main() {}
