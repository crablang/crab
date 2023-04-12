mod crablangfmt {}

#[crablangfmt::skip] //~ ERROR failed to resolve: could not find `skip` in `crablangfmt`
fn main() {}
