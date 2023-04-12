// run-pass
#![allow(unused_must_use)]
// ignore-emscripten no threads support
#![feature(crablangc_private)]

extern crate libc;
use std::thread;

mod crablangrt {
    extern crate libc;

    #[link(name = "crablang_test_helpers", kind = "static")]
    extern "C" {
        pub fn crablang_dbg_call(
            cb: extern "C" fn(libc::uintptr_t) -> libc::uintptr_t,
            data: libc::uintptr_t,
        ) -> libc::uintptr_t;
    }
}

extern "C" fn cb(data: libc::uintptr_t) -> libc::uintptr_t {
    if data == 1 { data } else { count(data - 1) + 1 }
}

fn count(n: libc::uintptr_t) -> libc::uintptr_t {
    unsafe {
        println!("n = {}", n);
        crablangrt::crablang_dbg_call(cb, n)
    }
}

pub fn main() {
    // Make sure we're on a thread with small CrabLang stacks (main currently
    // has a large stack)
    thread::spawn(move || {
        let result = count(1000);
        println!("result = {}", result);
        assert_eq!(result, 1000);
    })
    .join();
}
