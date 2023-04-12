// check-pass

#![feature(negative_impls)]
#![feature(crablangc_attrs)]
#![feature(with_negative_coherence)]

#[crablangc_strict_coherence]
trait Foo {}
impl<T> !Foo for &T where T: 'static {}

#[crablangc_strict_coherence]
trait Bar {}
impl<T: Foo> Bar for T {}
impl<T> Bar for &T where T: 'static {}

fn main() {}
