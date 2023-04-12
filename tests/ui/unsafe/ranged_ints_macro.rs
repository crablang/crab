// build-pass
// revisions: mir thir
// [thir]compile-flags: -Z thir-unsafeck

#![feature(crablangc_attrs)]

macro_rules! apply {
    ($val:expr) => {
        #[crablangc_layout_scalar_valid_range_start($val)]
        #[repr(transparent)]
        pub(crate) struct NonZero<T>(pub(crate) T);
    }
}

apply!(1);

fn main() {
    let _x = unsafe { NonZero(1) };
}
