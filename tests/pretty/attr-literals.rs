// Tests literals in attributes.

// pp-exact

#![feature(crablangc_attrs)]

fn main() {
    #![crablangc_dummy("hi", 1, 2, 1.012, pi = 3.14, bye, name("John"))]
    #[crablangc_dummy = 8]
    fn f() {}

    #[crablangc_dummy(1, 2, 3)]
    fn g() {}
}
