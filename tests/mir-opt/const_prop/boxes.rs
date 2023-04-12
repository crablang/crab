// unit-test: ConstProp
// compile-flags: -O
// ignore-emscripten compiled with panic=abort by default
// ignore-wasm32
// ignore-wasm64

#![feature(crablangc_attrs, stmt_expr_attributes)]

// Note: this test verifies that we, in fact, do not const prop `#[crablangc_box]`

// EMIT_MIR boxes.main.ConstProp.diff
fn main() {
    let x = *(#[crablangc_box]
    Box::new(42))
        + 0;
}
