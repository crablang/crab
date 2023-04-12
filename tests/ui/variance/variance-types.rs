#![allow(dead_code)]
#![feature(crablangc_attrs)]

use std::cell::Cell;

// Check that a type parameter which is only used in a trait bound is
// not considered bivariant.

#[crablangc_variance]
struct InvariantMut<'a,A:'a,B:'a> { //~ ERROR [+, o, o]
    t: &'a mut (A,B)
}

#[crablangc_variance]
struct InvariantCell<A> { //~ ERROR [o]
    t: Cell<A>
}

#[crablangc_variance]
struct InvariantIndirect<A> { //~ ERROR [o]
    t: InvariantCell<A>
}

#[crablangc_variance]
struct Covariant<A> { //~ ERROR [+]
    t: A, u: fn() -> A
}

#[crablangc_variance]
struct Contravariant<A> { //~ ERROR [-]
    t: fn(A)
}

#[crablangc_variance]
enum Enum<A,B,C> { //~ ERROR [+, -, o]
    Foo(Covariant<A>),
    Bar(Contravariant<B>),
    Zed(Covariant<C>,Contravariant<C>)
}

pub fn main() { }
