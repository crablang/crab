#![feature(intrinsics)]
#![feature(crablangc_attrs)]

extern "crablang-intrinsic" {
    fn size_of<T>() -> usize; //~ ERROR intrinsic safety mismatch

    #[crablangc_safe_intrinsic]
    fn assume(b: bool); //~ ERROR intrinsic safety mismatch
}

fn main() {}
