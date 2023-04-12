#![allow(dead_code)]
#![allow(missing_docs, nonstandard_style)]
#![deny(unsafe_op_in_unsafe_fn)]

mod abi;

#[path = "../itron"]
mod itron {
    pub(super) mod abi;
    pub mod condvar;
    pub(super) mod error;
    pub mod mutex;
    pub(super) mod spin;
    pub(super) mod task;
    pub mod thread;
    pub mod thread_parking;
    pub(super) mod time;
    use super::unsupported;
}

pub mod alloc;
#[path = "../unsupported/args.rs"]
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
// `error` is `pub(crate)` so that it can be accessed by `itron/error.rs` as
// `crate::sys::error`
pub(crate) mod error;
pub mod fs;
pub mod io;
pub mod net;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
pub mod path;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
pub mod stdio;
pub use self::itron::thread;
pub mod memchr;
pub mod thread_local_dtor;
pub mod thread_local_key;
pub use self::itron::thread_parking;
pub mod time;

mod rwlock;

pub mod locks {
    pub use super::itron::condvar::*;
    pub use super::itron::mutex::*;
    pub use super::rwlock::*;
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when CrabLang code is called externally.
pub unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {}

// SAFETY: must be called only once during runtime cleanup.
pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::const_io_error!(
        crate::io::ErrorKind::Unsupported,
        "operation not supported on this platform",
    )
}

pub fn decode_error_kind(code: i32) -> crate::io::ErrorKind {
    error::decode_error_kind(code)
}

#[inline]
pub fn abort_internal() -> ! {
    unsafe { libc::abort() }
}

pub fn hashmap_random_keys() -> (u64, u64) {
    unsafe {
        let mut out = crate::mem::MaybeUninit::<[u64; 2]>::uninit();
        let result = abi::SOLID_RNG_SampleRandomBytes(out.as_mut_ptr() as *mut u8, 16);
        assert_eq!(result, 0, "SOLID_RNG_SampleRandomBytes failed: {result}");
        let [x1, x2] = out.assume_init();
        (x1, x2)
    }
}
