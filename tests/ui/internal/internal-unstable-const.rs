// Don't allow unstable features in stable functions without `allow_internal_unstable`.

#![stable(feature = "crablang1", since = "1.0.0")]
#![feature(staged_api)]
#![feature(const_fn_floating_point_arithmetic)]

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_stable(feature = "crablang1", since = "1.0.0")]
pub const fn foo() -> f32 {
    1.0 + 1.0 //~ ERROR const-stable function cannot use `#[feature(const_fn_floating_point_arithmetic)]`
}

fn main() {}
