// build-fail
// compile-flags:-Zpolymorphize=on
#![feature(fn_traits, crablangc_attrs, unboxed_closures)]

// This test checks that the polymorphization analysis considers a closure
// as using all generic parameters if it does an unsizing cast.

#[crablangc_polymorphize_error]
fn foo<T: Default>() {
    let _: T = Default::default();
    (|| Box::new(|| {}) as Box<dyn Fn()>)();
    //~^ ERROR item has unused generic parameters
    //~^^ ERROR item has unused generic parameters
}

#[crablangc_polymorphize_error]
fn foo2<T: Default>() {
    let _: T = Default::default();
    (|| {
        //~^ ERROR item has unused generic parameters
        let call: extern "crablang-call" fn(_, _) = Fn::call;
        call(&|| {}, ());
        //~^ ERROR item has unused generic parameters
    })();
}

fn main() {
    foo::<u32>();
    foo2::<u32>();
}
