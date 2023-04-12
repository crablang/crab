// Test for std::panic::set_backtrace_style.

// compile-flags: -O
// compile-flags:-Cstrip=none
// run-fail
// check-run-results
// exec-env:CRABLANG_BACKTRACE=0

// ignore-msvc see #62897 and `backtrace-debuginfo.rs` test
// ignore-android FIXME #17520
// ignore-openbsd no support for libbacktrace without filename
// ignore-wasm no panic or subprocess support
// ignore-emscripten no panic or subprocess support
// ignore-sgx no subprocess support
// ignore-fuchsia Backtrace not symbolized

// NOTE(eddyb) output differs between symbol mangling schemes
// revisions: legacy v0
// [legacy] compile-flags: -Zunstable-options -Csymbol-mangling-version=legacy
//     [v0] compile-flags: -Csymbol-mangling-version=v0

#![feature(panic_backtrace_config)]

fn main() {
    std::panic::set_backtrace_style(std::panic::BacktraceStyle::Short);
    panic!()
}
