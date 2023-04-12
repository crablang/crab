/// Test for https://github.com/crablang/crablang-clippy/issues/2727

pub fn f(new: fn()) {
    new();
}

fn main() {}
