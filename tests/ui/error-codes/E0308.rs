#![feature(intrinsics)]
#![feature(crablangc_attrs)]

extern "crablang-intrinsic" {
    #[crablangc_safe_intrinsic]
    fn size_of<T>(); //~ ERROR E0308
}

fn main() {
}
