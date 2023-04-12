// run-pass

// https://github.com/crablang/crablang/issues/37448

fn main() {
    struct A;
    const FOO: &A = &(A as A);
    let _x = FOO;
}
