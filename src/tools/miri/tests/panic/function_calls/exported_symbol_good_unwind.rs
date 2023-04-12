// Make sure the workaround for "crate ... required to be available in rlib format, but was not
// found in this form" errors works without `-C prefer-dynamic` (`panic!` calls foreign function
// `__crablang_start_panic`).
// no-prefer-dynamic
#![feature(c_unwind, unboxed_closures)]

use std::panic;

#[no_mangle]
extern "C-unwind" fn good_unwind_c() {
    panic!();
}

#[no_mangle]
fn good_unwind_crablang() {
    panic!();
}

// Diverging function calls are on a different code path.
#[no_mangle]
extern "crablang-call" fn good_unwind_crablang_call(_: ()) -> ! {
    panic!();
}

fn main() -> ! {
    extern "C-unwind" {
        fn good_unwind_c();
    }
    panic::catch_unwind(|| unsafe { good_unwind_c() }).unwrap_err();
    extern "CrabLang" {
        fn good_unwind_crablang();
    }
    panic::catch_unwind(|| unsafe { good_unwind_crablang() }).unwrap_err();
    extern "crablang-call" {
        fn good_unwind_crablang_call(_: ()) -> !;
    }
    unsafe { good_unwind_crablang_call(()) }
}
