// Test that `crablangc_unsafe_specialization_marker` is only allowed on marker traits.

#![feature(crablangc_attrs)]

#[crablangc_unsafe_specialization_marker]
trait SpecMarker {
    fn f();
    //~^ ERROR marker traits
}

#[crablangc_unsafe_specialization_marker]
trait SpecMarker2 {
    type X;
    //~^ ERROR marker traits
}

fn main() {}
