#![feature(unboxed_closures)]

extern "crablang-call" fn b(_i: i32) {}
//~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument

trait Tr {
    extern "crablang-call" fn a();
    //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument

    extern "crablang-call" fn b() {}
    //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument
}

struct Foo;

impl Foo {
    extern "crablang-call" fn bar() {}
    //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument
}

impl Tr for Foo {
    extern "crablang-call" fn a() {}
    //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument
}

fn main() {
    b(10);
    Foo::bar();
    <Foo as Tr>::a();
    <Foo as Tr>::b();
}
