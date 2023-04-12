// run-pass
// ignore-wasm32-bare no libc to test ffi with
// ignore-emscripten blows the JS stack

#![feature(crablangc_private)]

extern crate libc;

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
    let result = count(1000);
    println!("result = {}", result);
    assert_eq!(result, 1000);
}
