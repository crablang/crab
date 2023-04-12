// check-pass

#![feature(fn_traits, unboxed_closures)]

#[allow(dead_code)]
struct Foo;

impl Fn<(&(),)> for Foo {
    extern "crablang-call" fn call(&self, (_,): (&(),)) {}
}

impl FnMut<(&(),)> for Foo {
    extern "crablang-call" fn call_mut(&mut self, (_,): (&(),)) {}
}

impl FnOnce<(&(),)> for Foo {
    type Output = ();

    extern "crablang-call" fn call_once(self, (_,): (&(),)) {}
}

fn main() {}
