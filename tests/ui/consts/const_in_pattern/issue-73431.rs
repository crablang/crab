// run-pass

// Regression test for https://github.com/crablang/crablang/issues/73431.

pub trait Zero {
    const ZERO: Self;
}

impl Zero for usize {
    const ZERO: Self = 0;
}

impl<T: Zero> Zero for Wrapper<T> {
    const ZERO: Self = Wrapper(T::ZERO);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Wrapper<T>(T);

fn is_zero(x: Wrapper<usize>) -> bool {
    match x {
        Zero::ZERO => true,
        _ => false,
    }
}

fn main() {
    let _ = is_zero(Wrapper(42));
}
