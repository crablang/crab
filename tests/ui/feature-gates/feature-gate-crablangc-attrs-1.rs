// Test that `#[crablangc_*]` attributes are gated by `crablangc_attrs` feature gate.

#[crablangc_variance] //~ ERROR the `#[crablangc_variance]` attribute is just used for crablangc unit tests and will never be stable
#[crablangc_error] //~ ERROR the `#[crablangc_error]` attribute is just used for crablangc unit tests and will never be stable
#[crablangc_nonnull_optimization_guaranteed] //~ ERROR the `#[crablangc_nonnull_optimization_guaranteed]` attribute is just used to enable niche optimizations in libcore and libstd and will never be stable

fn main() {}
