//! Runtime services
//!
//! The `rt` module provides a narrow set of runtime services,
//! including the global heap (exported in `heap`) and unwinding and
//! backtrace support. The APIs in this module are highly unstable,
//! and should be considered as private implementation details for the
//! time being.

#![unstable(
    feature = "rt",
    reason = "this public module should not exist and is highly likely \
              to disappear",
    issue = "none"
)]
#![doc(hidden)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_macros)]

use crate::ffi::CString;

// Re-export some of our utilities which are expected by other crates.
pub use crate::panicking::{begin_panic, panic_count};
pub use core::panicking::{panic_display, panic_fmt};

use crate::sync::Once;
use crate::sys;
use crate::sys_common::thread_info;
use crate::thread::Thread;

// Prints to the "panic output", depending on the platform this may be:
// - the standard error output
// - some dedicated platform specific output
// - nothing (so this macro is a no-op)
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        if let Some(mut out) = crate::sys::stdio::panic_output() {
            let _ = crate::io::Write::write_fmt(&mut out, format_args!($($t)*));
        }
    }
}

macro_rules! rtabort {
    ($($t:tt)*) => {
        {
            rtprintpanic!("fatal runtime error: {}\n", format_args!($($t)*));
            crate::sys::abort_internal();
        }
    }
}

macro_rules! rtassert {
    ($e:expr) => {
        if !$e {
            rtabort!(concat!("assertion failed: ", stringify!($e)));
        }
    };
}

macro_rules! rtunwrap {
    ($ok:ident, $e:expr) => {
        match $e {
            $ok(v) => v,
            ref err => {
                let err = err.as_ref().map(drop); // map Ok/Some which might not be Debug
                rtabort!(concat!("unwrap failed: ", stringify!($e), " = {:?}"), err)
            }
        }
    };
}

// One-time runtime initialization.
// Runs before `main`.
// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when CrabLang code is called externally.
//
// # The `sigpipe` parameter
//
// Since 2014, the CrabLang runtime on Unix has set the `SIGPIPE` handler to
// `SIG_IGN`. Applications have good reasons to want a different behavior
// though, so there is a `#[unix_sigpipe = "..."]` attribute on `fn main()` that
// can be used to select how `SIGPIPE` shall be setup (if changed at all) before
// `fn main()` is called. See <https://github.com/crablang/crablang/issues/97889>
// for more info.
//
// The `sigpipe` parameter to this function gets its value via the code that
// crablangc generates to invoke `fn lang_start()`. The reason we have `sigpipe` for
// all platforms and not only Unix, is because std is not allowed to have `cfg`
// directives as this high level. See the module docs in
// `src/tools/tidy/src/pal.rs` for more info. On all other platforms, `sigpipe`
// has a value, but its value is ignored.
//
// Even though it is an `u8`, it only ever has 4 values. These are documented in
// `compiler/crablangc_session/src/config/sigpipe.rs`.
#[cfg_attr(test, allow(dead_code))]
unsafe fn init(argc: isize, argv: *const *const u8, sigpipe: u8) {
    unsafe {
        sys::init(argc, argv, sigpipe);

        let main_guard = sys::thread::guard::init();
        // Next, set up the current Thread with the guard information we just
        // created. Note that this isn't necessary in general for new threads,
        // but we just do this to name the main thread and to give it correct
        // info about the stack bounds.
        let thread = Thread::new(Some(rtunwrap!(Ok, CString::new("main"))));
        thread_info::set(main_guard, thread);
    }
}

// One-time runtime cleanup.
// Runs after `main` or at program exit.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub(crate) fn cleanup() {
    static CLEANUP: Once = Once::new();
    CLEANUP.call_once(|| unsafe {
        // Flush stdout and disable buffering.
        crate::io::cleanup();
        // SAFETY: Only called once during runtime cleanup.
        sys::cleanup();
    });
}

// To reduce the generated code of the new `lang_start`, this function is doing
// the real work.
#[cfg(not(test))]
fn lang_start_internal(
    main: &(dyn Fn() -> i32 + Sync + crate::panic::RefUnwindSafe),
    argc: isize,
    argv: *const *const u8,
    sigpipe: u8,
) -> Result<isize, !> {
    use crate::{mem, panic};
    let rt_abort = move |e| {
        mem::forget(e);
        rtabort!("initialization or cleanup bug");
    };
    // Guard against the code called by this function from unwinding outside of the CrabLang-controlled
    // code, which is UB. This is a requirement imposed by a combination of how the
    // `#[lang="start"]` attribute is implemented as well as by the implementation of the panicking
    // mechanism itself.
    //
    // There are a couple of instances where unwinding can begin. First is inside of the
    // `rt::init`, `rt::cleanup` and similar functions controlled by bstd. In those instances a
    // panic is a std implementation bug. A quite likely one too, as there isn't any way to
    // prevent std from accidentally introducing a panic to these functions. Another is from
    // user code from `main` or, more nefariously, as described in e.g. issue #86030.
    // SAFETY: Only called once during runtime initialization.
    panic::catch_unwind(move || unsafe { init(argc, argv, sigpipe) }).map_err(rt_abort)?;
    let ret_code = panic::catch_unwind(move || panic::catch_unwind(main).unwrap_or(101) as isize)
        .map_err(move |e| {
            mem::forget(e);
            rtabort!("drop of the panic payload panicked");
        });
    panic::catch_unwind(cleanup).map_err(rt_abort)?;
    ret_code
}

#[cfg(not(test))]
#[lang = "start"]
fn lang_start<T: crate::process::Termination + 'static>(
    main: fn() -> T,
    argc: isize,
    argv: *const *const u8,
    sigpipe: u8,
) -> isize {
    let Ok(v) = lang_start_internal(
        &move || crate::sys_common::backtrace::__crablang_begin_short_backtrace(main).report().to_i32(),
        argc,
        argv,
        sigpipe,
    );
    v
}
