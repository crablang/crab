// check-pass

// See https://github.com/crablang/crablang/issues/95151
#[track_caller]
macro_rules! _foo {
    () => {};
}

fn main() {
}
