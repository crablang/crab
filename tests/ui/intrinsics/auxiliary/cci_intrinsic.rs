#![feature(intrinsics)]

pub mod crablangi {
    extern "crablang-intrinsic" {
        pub fn atomic_xchg_seqcst<T>(dst: *mut T, src: T) -> T;
    }
}

#[inline(always)]
pub fn atomic_xchg_seqcst(dst: *mut isize, src: isize) -> isize {
    unsafe {
        crablangi::atomic_xchg_seqcst(dst, src)
    }
}
