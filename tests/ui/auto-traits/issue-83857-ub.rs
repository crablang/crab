#![allow(suspicious_auto_trait_impls)]

struct Always<T, U>(T, U);
unsafe impl<T, U> Send for Always<T, U> {}
struct Foo<T, U>(Always<T, U>);

trait False {}
unsafe impl<U: False> Send for Foo<u32, U> {}

trait WithAssoc {
    type Output;
}
impl<T: Send> WithAssoc for T {
    type Output = Self;
}
impl WithAssoc for Foo<u32, ()> {
    type Output = Box<i32>;
}

fn generic<T, U>(v: Foo<T, U>, f: fn(<Foo<T, U> as WithAssoc>::Output) -> i32) {
    //~^ ERROR `Foo<T, U>` cannot be sent between threads safely
    f(foo(v));
}

fn foo<T: Send>(x: T) -> <T as WithAssoc>::Output {
    x
}

fn main() {
    generic(Foo(Always(0, ())), |b| *b);
}
