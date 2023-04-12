// Regression test for https://github.com/crablang/crablang-clippy/issues/5207

pub async fn bar<'a, T: 'a>(_: T) {}

fn main() {}
