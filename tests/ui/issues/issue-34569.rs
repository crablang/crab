// run-pass
// compile-flags:-g
// ignore-asmjs wasm2js does not support source maps yet

// In this test we just want to make sure that the code below does not lead to
// a debuginfo verification assertion during compilation. This was caused by the
// closure in the guard being codegened twice due to how match expressions are
// handled.
//
// See https://github.com/crablang/crablang/issues/34569 for details.

fn main() {
    match 0 {
        e if (|| { e == 0 })() => {},
        1 => {},
        _ => {}
    }
}
