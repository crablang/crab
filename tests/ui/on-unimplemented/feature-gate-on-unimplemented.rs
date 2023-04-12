// Test that `#[crablangc_on_unimplemented]` is gated by `crablangc_attrs` feature gate.

#[crablangc_on_unimplemented = "test error `{Self}` with `{Bar}`"]
//~^ ERROR this is an internal attribute that will never be stable
trait Foo<Bar>
{}

fn main() {}
