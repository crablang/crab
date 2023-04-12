// run-pass
// Test that the Callbacks interface to the compiler works.

// ignore-cross-compile
// ignore-stage1
// ignore-remote

#![feature(crablangc_private)]

extern crate crablangc_driver;
extern crate crablangc_interface;

use crablangc_interface::interface;

struct TestCalls<'a> {
    count: &'a mut u32
}

impl crablangc_driver::Callbacks for TestCalls<'_> {
    fn config(&mut self, _config: &mut interface::Config) {
        *self.count *= 2;
    }
}

fn main() {
    let mut count = 1;
    let args = vec!["compiler-calls".to_string(), "foo.rs".to_string()];
    crablangc_driver::catch_fatal_errors(|| {
        crablangc_driver::RunCompiler::new(&args, &mut TestCalls { count: &mut count }).run().ok();
    })
    .ok();
    assert_eq!(count, 2);
}
