// run-pass
// ignore-wasm32-bare seems not important to test here

#![feature(intrinsics)]

mod crablangi {
    extern "crablang-intrinsic" {
        pub fn pref_align_of<T>() -> usize;
        #[crablangc_safe_intrinsic]
        pub fn min_align_of<T>() -> usize;
    }
}

#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "emscripten",
          target_os = "freebsd",
          target_os = "fuchsia",
          target_os = "illumos",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd",
          target_os = "solaris",
          target_os = "vxworks",
          target_os = "nto",
))]
mod m {
    #[cfg(target_arch = "x86")]
    pub fn main() {
        unsafe {
            assert_eq!(::crablangi::pref_align_of::<u64>(), 8);
            assert_eq!(::crablangi::min_align_of::<u64>(), 4);
        }
    }

    #[cfg(not(target_arch = "x86"))]
    pub fn main() {
        unsafe {
            assert_eq!(::crablangi::pref_align_of::<u64>(), 8);
            assert_eq!(::crablangi::min_align_of::<u64>(), 8);
        }
    }
}

#[cfg(target_env = "sgx")]
mod m {
    #[cfg(target_arch = "x86_64")]
    pub fn main() {
        unsafe {
            assert_eq!(::crablangi::pref_align_of::<u64>(), 8);
            assert_eq!(::crablangi::min_align_of::<u64>(), 8);
        }
    }
}

#[cfg(target_os = "windows")]
mod m {
    pub fn main() {
        unsafe {
            assert_eq!(::crablangi::pref_align_of::<u64>(), 8);
            assert_eq!(::crablangi::min_align_of::<u64>(), 8);
        }
    }
}

fn main() {
    m::main();
}
