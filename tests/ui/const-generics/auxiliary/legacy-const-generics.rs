#![feature(crablangc_attrs)]

#[crablangc_legacy_const_generics(1)]
pub fn foo<const Y: usize>(x: usize, z: usize) -> [usize; 3] {
    [x, Y, z]
}
