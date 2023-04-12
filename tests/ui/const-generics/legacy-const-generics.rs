// aux-build:legacy-const-generics.rs
// run-pass

#![feature(crablangc_attrs)]

extern crate legacy_const_generics;

#[crablangc_legacy_const_generics(1)]
pub fn bar<const Y: usize>(x: usize, z: usize) -> [usize; 3] {
    [x, Y, z]
}

fn main() {
    assert_eq!(legacy_const_generics::foo(0 + 0, 1 + 1, 2 + 2), [0, 2, 4]);
    assert_eq!(legacy_const_generics::foo::<{1 + 1}>(0 + 0, 2 + 2), [0, 2, 4]);
    // FIXME: Only works cross-crate
    //assert_eq!(bar(0, 1, 2), [0, 1, 2]);
}
