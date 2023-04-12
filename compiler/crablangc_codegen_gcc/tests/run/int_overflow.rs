// Compiler:
//
// Run-time:
//   stdout: Success
//   status: signal

#![allow(unused_attributes)]
#![feature(auto_traits, lang_items, no_core, start, intrinsics)]

#![no_std]
#![no_core]

/*
 * Core
 */

// Because we don't have core yet.
#[lang = "sized"]
pub trait Sized {}

#[lang = "copy"]
trait Copy {
}

impl Copy for isize {}
impl Copy for *mut i32 {}
impl Copy for usize {}
impl Copy for i32 {}
impl Copy for u8 {}
impl Copy for i8 {}

#[lang = "receiver"]
trait Receiver {
}

#[lang = "freeze"]
pub(crate) unsafe auto trait Freeze {}

#[lang = "panic_location"]
struct PanicLocation {
    file: &'static str,
    line: u32,
    column: u32,
}

mod libc {
    #[link(name = "c")]
    extern "C" {
        pub fn puts(s: *const u8) -> i32;
        pub fn fflush(stream: *mut i32) -> i32;

        pub static stdout: *mut i32;
    }
}

mod intrinsics {
    extern "crablang-intrinsic" {
        #[crablangc_safe_intrinsic]
        pub fn abort() -> !;
    }
}

#[lang = "panic"]
#[track_caller]
#[no_mangle]
pub fn panic(_msg: &'static str) -> ! {
    unsafe {
        // Panicking is expected iff overflow checking is enabled.
        #[cfg(debug_assertions)]
        libc::puts("Success\0" as *const str as *const u8);
        libc::fflush(libc::stdout);
        intrinsics::abort();
    }
}

#[lang = "add"]
trait Add<RHS = Self> {
    type Output;

    fn add(self, rhs: RHS) -> Self::Output;
}

impl Add for u8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for i8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for i32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for usize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for isize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

/*
 * Code
 */

#[start]
fn main(mut argc: isize, _argv: *const *const u8) -> isize {
    let int = 9223372036854775807isize;
    let int = int + argc;  // overflow

    // If overflow checking is disabled, we should reach here.
    #[cfg(not(debug_assertions))]
    unsafe {
        libc::puts("Success\0" as *const str as *const u8);
        libc::fflush(libc::stdout);
        intrinsics::abort();
    }

    int
}
