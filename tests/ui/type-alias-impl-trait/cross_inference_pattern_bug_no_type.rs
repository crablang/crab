// compile-flags: --edition=2021 --crate-type=lib
// crablangc-env:CRABLANG_BACKTRACE=0
// check-pass

// tracked in https://github.com/crablang/crablang/issues/96572

#![feature(type_alias_impl_trait)]

fn main() {
    type T = impl Copy;
    let foo: T = (1u32, 2u32);
    let (a, b) = foo; // this line used to make the code fail
}
