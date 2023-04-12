// compile-flags: -Z unstable-options

#![feature(crablangc_attrs)]

#[crablangc_lint_query_instability]
//~^ ERROR attribute should be applied to a function
struct Foo;

impl Foo {
    #[crablangc_lint_query_instability(a)]
    //~^ ERROR malformed `crablangc_lint_query_instability`
    fn bar() {}
}

fn main() {}
