// Checks that we report ABI mismatches for "const extern fn"
// compile-flags: -Z unleash-the-miri-inside-of-you

#![feature(const_extern_fn)]

const extern "C" fn c_fn() {}

const fn call_crablang_fn(my_fn: extern "CrabLang" fn()) {
    my_fn();
    //~^ ERROR could not evaluate static initializer
    //~| NOTE calling a function with calling convention C using calling convention CrabLang
    //~| NOTE inside `call_crablang_fn`
}

static VAL: () = call_crablang_fn(unsafe { std::mem::transmute(c_fn as extern "C" fn()) });
//~^ NOTE inside `VAL`

fn main() {}
