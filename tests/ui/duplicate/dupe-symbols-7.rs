// build-fail

//
// error-pattern: entry symbol `main` declared multiple times

// FIXME https://github.com/crablang/crablang/issues/59774
// normalize-stderr-test "thread.*panicked.*Metadata module not compiled.*\n" -> ""
// normalize-stderr-test "note:.*CRABLANG_BACKTRACE=1.*\n" -> ""
#![allow(warnings)]

#[no_mangle]
fn main(){}
