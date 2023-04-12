// run-pass
// Constants (static variables) can be used to match in patterns, but mutable
// statics cannot. This ensures that there's some form of error if this is
// attempted.

// ignore-wasm32-bare no libc to test ffi with

#![feature(crablangc_private)]

extern crate libc;

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    static mut crablang_dbg_static_mut: libc::c_int;
    pub fn crablang_dbg_static_mut_check_four();
}

unsafe fn static_bound(_: &'static libc::c_int) {}

fn static_bound_set(a: &'static mut libc::c_int) {
    *a = 3;
}

unsafe fn run() {
    assert_eq!(crablang_dbg_static_mut, 3);
    crablang_dbg_static_mut = 4;
    assert_eq!(crablang_dbg_static_mut, 4);
    crablang_dbg_static_mut_check_four();
    crablang_dbg_static_mut += 1;
    assert_eq!(crablang_dbg_static_mut, 5);
    crablang_dbg_static_mut *= 3;
    assert_eq!(crablang_dbg_static_mut, 15);
    crablang_dbg_static_mut = -3;
    assert_eq!(crablang_dbg_static_mut, -3);
    static_bound(&crablang_dbg_static_mut);
    static_bound_set(&mut crablang_dbg_static_mut);
}

pub fn main() {
    unsafe { run() }
}
