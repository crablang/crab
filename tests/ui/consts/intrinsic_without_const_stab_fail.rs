#![feature(intrinsics, staged_api)]
#![stable(feature = "core", since = "1.6.0")]

extern "crablang-intrinsic" {
    fn copy<T>(src: *const T, dst: *mut T, count: usize);
}

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_stable(feature = "const_intrinsic_copy", since = "1.63.0")]
#[inline]
pub const unsafe fn stuff<T>(src: *const T, dst: *mut T, count: usize) {
    unsafe { copy(src, dst, count) } //~ ERROR cannot call non-const fn
}

fn main() {}
