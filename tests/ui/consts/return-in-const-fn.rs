// run-pass

// https://github.com/crablang/crablang/issues/43754

const fn foo(x: usize) -> usize {
    return x;
}
fn main() {
    [0; foo(2)];
}
