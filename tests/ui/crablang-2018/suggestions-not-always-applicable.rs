// aux-build:suggestions-not-always-applicable.rs
// edition:2015
// run-crablangfix
// crablangfix-only-machine-applicable
// check-pass

#![feature(crablang_2018_preview)]
#![warn(crablang_2018_compatibility)]

extern crate suggestions_not_always_applicable as foo;

pub struct Foo;

mod test {
    use crate::foo::foo;

    #[foo]
    fn main() {}
}

fn main() {
    test::foo();
}
