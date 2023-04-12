#![feature(crablangc_attrs)]
#![feature(type_alias_impl_trait)]

type Foo = impl Fn() -> usize;
const fn bar() -> Foo {
    || 0usize
}
const BAZR: Foo = bar();

#[crablangc_error]
fn main() {} //~ ERROR
