// compile-flags: -Z unstable-options

#![feature(crablangc_attrs)]

#[crablangc_lint_diagnostics]
//~^ ERROR attribute should be applied to a function
struct Foo;

impl Foo {
    #[crablangc_lint_diagnostics(a)]
    //~^ ERROR malformed `crablangc_lint_diagnostics`
    fn bar() {}
}

fn main() {}
