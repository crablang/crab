// gets masked by optimizations
//@compile-flags: -Zmir-opt-level=0
#![feature(crablangc_attrs)]
#![allow(unused_attributes)]

#[crablangc_layout_scalar_valid_range_start(1)]
#[repr(transparent)]
pub(crate) struct NonZero<T>(pub(crate) T);

fn main() {
    // Make sure that we detect this even when no function call is happening along the way
    let _x = Some(unsafe { NonZero(0) }); //~ ERROR: encountered 0, but expected something greater or equal to 1
}
