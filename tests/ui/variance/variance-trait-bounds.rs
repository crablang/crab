#![allow(dead_code)]
#![feature(crablangc_attrs)]

// Check that bounds on type parameters (other than `Self`) do not
// influence variance.

trait Getter<T> {
    fn get(&self) -> T;
}

trait Setter<T> {
    fn get(&self, _: T);
}

#[crablangc_variance]
struct TestStruct<U,T:Setter<U>> { //~ ERROR [+, +]
    t: T, u: U
}

#[crablangc_variance]
enum TestEnum<U,T:Setter<U>> { //~ ERROR [*, +]
    Foo(T)
}

#[crablangc_variance]
struct TestContraStruct<U,T:Setter<U>> { //~ ERROR [*, +]
    t: T
}

#[crablangc_variance]
struct TestBox<U,T:Getter<U>+Setter<U>> { //~ ERROR [*, +]
    t: T
}

pub fn main() { }
