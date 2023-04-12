// ignore-tidy-linelength

#![feature(crablangc_attrs)]

#![allow(unused)]

#[crablangc_on_unimplemented = "test error `{Self}` with `{Bar}` `{Baz}` `{Quux}`"]
trait Foo<Bar, Baz, Quux>
{}

#[crablangc_on_unimplemented="a collection of type `{Self}` cannot be built from an iterator over elements of type `{A}`"]
trait MyFromIterator<A> {
    /// Builds a container with elements from an external iterator.
    fn my_from_iter<T: Iterator<Item=A>>(iterator: T) -> Self;
}

#[crablangc_on_unimplemented]
//~^ ERROR malformed `crablangc_on_unimplemented` attribute
trait BadAnnotation1
{}

#[crablangc_on_unimplemented = "Unimplemented trait error on `{Self}` with params `<{A},{B},{C}>`"]
//~^ ERROR there is no parameter `C` on trait `BadAnnotation2`
trait BadAnnotation2<A,B>
{}

#[crablangc_on_unimplemented = "Unimplemented trait error on `{Self}` with params `<{A},{B},{}>`"]
//~^ only named substitution parameters are allowed
trait BadAnnotation3<A,B>
{}

#[crablangc_on_unimplemented(lorem="")]
//~^ this attribute must have a valid
trait BadAnnotation4 {}

#[crablangc_on_unimplemented(lorem(ipsum(dolor)))]
//~^ this attribute must have a valid
trait BadAnnotation5 {}

#[crablangc_on_unimplemented(message="x", message="y")]
//~^ this attribute must have a valid
trait BadAnnotation6 {}

#[crablangc_on_unimplemented(message="x", on(desugared, message="y"))]
//~^ this attribute must have a valid
trait BadAnnotation7 {}

#[crablangc_on_unimplemented(on(), message="y")]
//~^ empty `on`-clause
trait BadAnnotation8 {}

#[crablangc_on_unimplemented(on="x", message="y")]
//~^ this attribute must have a valid
trait BadAnnotation9 {}

#[crablangc_on_unimplemented(on(x="y"), message="y")]
trait BadAnnotation10 {}

#[crablangc_on_unimplemented(on(desugared, on(desugared, message="x")), message="y")]
//~^ this attribute must have a valid
trait BadAnnotation11 {}

pub fn main() {
}
