// revisions: no_drop_tracking drop_tracking drop_tracking_mir
// [drop_tracking] compile-flags: -Zdrop-tracking
// [drop_tracking_mir] compile-flags: -Zdrop-tracking-mir
#![feature(generators)]
#![feature(auto_traits)]
#![feature(negative_impls)]

auto trait Foo {}

struct No;

impl !Foo for No {}

struct A<'a, 'b>(&'a mut bool, &'b mut bool, No);

impl<'a, 'b: 'a> Foo for A<'a, 'b> {}

struct OnlyFooIfStaticRef(No);
impl Foo for &'static OnlyFooIfStaticRef {}

struct OnlyFooIfRef(No);
impl<'a> Foo for &'a OnlyFooIfRef {}

fn assert_foo<T: Foo>(f: T) {}

fn main() {
    // Make sure 'static is erased for generator interiors so we can't match it in trait selection
    let x: &'static _ = &OnlyFooIfStaticRef(No);
    let gen = move || {
        let x = x;
        yield;
        assert_foo(x);
    };
    assert_foo(gen);
    //~^ ERROR implementation of `Foo` is not general enough

    // Allow impls which matches any lifetime
    let x = &OnlyFooIfRef(No);
    let gen = move || {
        let x = x;
        yield;
        assert_foo(x);
    };
    assert_foo(gen); // ok

    // Disallow impls which relates lifetimes in the generator interior
    let gen = move || {
        let a = A(&mut true, &mut true, No);
        //~^ temporary value dropped while borrowed
        //~| temporary value dropped while borrowed
        yield;
        assert_foo(a);
    };
    assert_foo(gen);
    //~^ ERROR not general enough
}
