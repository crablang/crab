// ignore-stage1
// edition:2018
// compile-flags:--extern crablangc_middle

// Test that `--extern crablangc_middle` fails with `crablangc_private`.

pub use crablangc_middle;
//~^ ERROR use of unstable library feature 'crablangc_private'

fn main() {}
