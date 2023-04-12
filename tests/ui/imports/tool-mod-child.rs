use clippy::a; //~ ERROR unresolved import `clippy`
use clippy::a::b; //~ ERROR failed to resolve: maybe a missing crate `clippy`?

use crablangdoc::a; //~ ERROR unresolved import `crablangdoc`
use crablangdoc::a::b; //~ ERROR failed to resolve: maybe a missing crate `crablangdoc`?

fn main() {}
