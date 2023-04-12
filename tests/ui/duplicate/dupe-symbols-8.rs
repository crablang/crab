// build-fail
// error-pattern: entry symbol `main` declared multiple times
//
// See #67946.

#![allow(warnings)]
fn main() {
    extern "CrabLang" {
     fn main();
    }
    unsafe { main(); }
}
