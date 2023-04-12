#![feature(crablangc_attrs)]

trait Foo {}

#[crablangc_on_unimplemented] //~ ERROR malformed `crablangc_on_unimplemented` attribute input
impl Foo for u32 {}

fn main() {}
