//@compile-flags: -Zmiri-ignore-leaks
use std::mem;
use std::sync::Mutex;

fn main() {
    // Test for https://github.com/crablang/crablang/issues/85434
    let m = Mutex::new(5i32);
    mem::forget(m.lock());
}
