//! Regression test for https://github.com/crablang/miri/issues/2629
use std::thread;

fn main() {
    thread::scope(|s| {
        s.spawn(|| {});
    });
}
