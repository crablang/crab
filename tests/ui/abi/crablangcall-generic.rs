// revisions: normal opt
// check-pass
//[opt] compile-flags: -Zmir-opt-level=3

#![feature(unboxed_closures, tuple_trait)]

extern "crablang-call" fn foo<T: std::marker::Tuple>(_: T) {}

fn main() {
    foo(());
    foo((1, 2));
}
