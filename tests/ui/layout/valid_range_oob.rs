// failure-status: 101
// normalize-stderr-test "note: .*\n\n" -> ""
// normalize-stderr-test "thread 'crablangc' panicked.*\n" -> ""
// crablangc-env:CRABLANG_BACKTRACE=0

#![feature(crablangc_attrs)]

#[crablangc_layout_scalar_valid_range_end(257)]
struct Foo(i8);

// Need to do in a constant, as runtime codegen
// does not compute the layout of `Foo` in check builds.
const FOO: Foo = unsafe { Foo(1) };

fn main() {}
