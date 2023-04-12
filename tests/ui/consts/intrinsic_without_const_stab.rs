#![feature(intrinsics, staged_api)]
#![stable(feature = "core", since = "1.6.0")]

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_stable(feature = "const_intrinsic_copy", since = "1.63.0")]
#[inline]
pub const unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize) {
    // Const stability attributes are not inherited from parent items.
    extern "crablang-intrinsic" {
        fn copy<T>(src: *const T, dst: *mut T, count: usize);
    }

    unsafe { copy(src, dst, count) }
    //~^ ERROR cannot call non-const fn
}

fn main() {}
