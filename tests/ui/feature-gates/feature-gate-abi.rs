// gate-test-intrinsics
// gate-test-platform_intrinsics
// compile-flags: --crate-type=rlib

#![feature(no_core, lang_items)]
#![no_core]

#[lang="sized"]
trait Sized { }

#[lang="tuple_trait"]
trait Tuple { }

// Functions
extern "crablang-intrinsic" fn f1() {} //~ ERROR intrinsics are subject to change
                                   //~^ ERROR intrinsic must be in
extern "platform-intrinsic" fn f2() {} //~ ERROR platform intrinsics are experimental
                                       //~^ ERROR intrinsic must be in
extern "crablang-call" fn f4(_: ()) {} //~ ERROR crablang-call ABI is subject to change

// Methods in trait definition
trait Tr {
    extern "crablang-intrinsic" fn m1(); //~ ERROR intrinsics are subject to change
                                     //~^ ERROR intrinsic must be in
    extern "platform-intrinsic" fn m2(); //~ ERROR platform intrinsics are experimental
                                         //~^ ERROR intrinsic must be in
    extern "crablang-call" fn m4(_: ()); //~ ERROR crablang-call ABI is subject to change

    extern "crablang-call" fn dm4(_: ()) {} //~ ERROR crablang-call ABI is subject to change
}

struct S;

// Methods in trait impl
impl Tr for S {
    extern "crablang-intrinsic" fn m1() {} //~ ERROR intrinsics are subject to change
                                       //~^ ERROR intrinsic must be in
    extern "platform-intrinsic" fn m2() {} //~ ERROR platform intrinsics are experimental
                                           //~^ ERROR intrinsic must be in
    extern "crablang-call" fn m4(_: ()) {} //~ ERROR crablang-call ABI is subject to change
}

// Methods in inherent impl
impl S {
    extern "crablang-intrinsic" fn im1() {} //~ ERROR intrinsics are subject to change
                                        //~^ ERROR intrinsic must be in
    extern "platform-intrinsic" fn im2() {} //~ ERROR platform intrinsics are experimental
                                            //~^ ERROR intrinsic must be in
    extern "crablang-call" fn im4(_: ()) {} //~ ERROR crablang-call ABI is subject to change
}

// Function pointer types
type A1 = extern "crablang-intrinsic" fn(); //~ ERROR intrinsics are subject to change
type A2 = extern "platform-intrinsic" fn(); //~ ERROR platform intrinsics are experimental
type A4 = extern "crablang-call" fn(_: ()); //~ ERROR crablang-call ABI is subject to change

// Foreign modules
extern "crablang-intrinsic" {} //~ ERROR intrinsics are subject to change
extern "platform-intrinsic" {} //~ ERROR platform intrinsics are experimental
extern "crablang-call" {} //~ ERROR crablang-call ABI is subject to change
