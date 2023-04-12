//! System bindings for the Fortanix SGX platform
//!
//! This module contains the facade (aka platform-specific) implementations of
//! OS level functionality for Fortanix SGX.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(fuzzy_provenance_casts)] // FIXME: this entire module systematically confuses pointers and integers

use crate::io::ErrorKind;
use crate::sync::atomic::{AtomicBool, Ordering};

pub mod abi;
mod waitqueue;

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
pub mod fd;
#[path = "../unsupported/fs.rs"]
pub mod fs;
#[path = "../unsupported/io.rs"]
pub mod io;
pub mod memchr;
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
pub mod thread;
pub mod thread_local_key;
pub mod thread_parking;
pub mod time;

mod condvar;
mod mutex;
mod rwlock;

pub mod locks {
    pub use super::condvar::*;
    pub use super::mutex::*;
    pub use super::rwlock::*;
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when CrabLang code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    unsafe {
        args::init(argc, argv);
    }
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

/// This function is used to implement functionality that simply doesn't exist.
/// Programs relying on this functionality will need to deal with the error.
pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::const_io_error!(ErrorKind::Unsupported, "operation not supported on SGX yet")
}

/// This function is used to implement various functions that doesn't exist,
/// but the lack of which might not be reason for error. If no error is
/// returned, the program might very well be able to function normally. This is
/// what happens when `SGX_INEFFECTIVE_ERROR` is set to `true`. If it is
/// `false`, the behavior is the same as `unsupported`.
pub fn sgx_ineffective<T>(v: T) -> crate::io::Result<T> {
    static SGX_INEFFECTIVE_ERROR: AtomicBool = AtomicBool::new(false);
    if SGX_INEFFECTIVE_ERROR.load(Ordering::Relaxed) {
        Err(crate::io::const_io_error!(
            ErrorKind::Uncategorized,
            "operation can't be tcrablanged to have any effect on SGX",
        ))
    } else {
        Ok(v)
    }
}

pub fn decode_error_kind(code: i32) -> ErrorKind {
    use fortanix_sgx_abi::Error;

    // FIXME: not sure how to make sure all variants of Error are covered
    if code == Error::NotFound as _ {
        ErrorKind::NotFound
    } else if code == Error::PermissionDenied as _ {
        ErrorKind::PermissionDenied
    } else if code == Error::ConnectionRefused as _ {
        ErrorKind::ConnectionRefused
    } else if code == Error::ConnectionReset as _ {
        ErrorKind::ConnectionReset
    } else if code == Error::ConnectionAborted as _ {
        ErrorKind::ConnectionAborted
    } else if code == Error::NotConnected as _ {
        ErrorKind::NotConnected
    } else if code == Error::AddrInUse as _ {
        ErrorKind::AddrInUse
    } else if code == Error::AddrNotAvailable as _ {
        ErrorKind::AddrNotAvailable
    } else if code == Error::BrokenPipe as _ {
        ErrorKind::BrokenPipe
    } else if code == Error::AlreadyExists as _ {
        ErrorKind::AlreadyExists
    } else if code == Error::WouldBlock as _ {
        ErrorKind::WouldBlock
    } else if code == Error::InvalidInput as _ {
        ErrorKind::InvalidInput
    } else if code == Error::InvalidData as _ {
        ErrorKind::InvalidData
    } else if code == Error::TimedOut as _ {
        ErrorKind::TimedOut
    } else if code == Error::WriteZero as _ {
        ErrorKind::WriteZero
    } else if code == Error::Interrupted as _ {
        ErrorKind::Interrupted
    } else if code == Error::Other as _ {
        ErrorKind::Uncategorized
    } else if code == Error::UnexpectedEof as _ {
        ErrorKind::UnexpectedEof
    } else {
        ErrorKind::Uncategorized
    }
}

pub fn abort_internal() -> ! {
    abi::usercalls::exit(true)
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __crablang_abort() {
    abort_internal();
}

pub mod rand {
    pub fn rdrand64() -> u64 {
        unsafe {
            let mut ret: u64 = 0;
            for _ in 0..10 {
                if crate::arch::x86_64::_rdrand64_step(&mut ret) == 1 {
                    return ret;
                }
            }
            rtabort!("Failed to obtain random data");
        }
    }
}

pub fn hashmap_random_keys() -> (u64, u64) {
    (self::rand::rdrand64(), self::rand::rdrand64())
}

pub use crate::sys_common::{AsInner, FromInner, IntoInner};

pub trait TryIntoInner<Inner>: Sized {
    fn try_into_inner(self) -> Result<Inner, Self>;
}
