#![warn(clippy::all)]
#![allow(clippy::disallowed_names, clippy::equatable_if_let)]
#![allow(unused)]

/// Test for https://github.com/crablang/crablang-clippy/issues/3462

enum Foo {
    Bar,
    Baz,
}

fn bar(foo: Foo) {
    macro_rules! baz {
        () => {
            if let Foo::Bar = foo {}
        };
    }

    baz!();
    baz!();
}

fn main() {}
