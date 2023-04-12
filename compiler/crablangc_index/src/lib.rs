#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]
#![cfg_attr(
    feature = "nightly",
    feature(
        allow_internal_unstable,
        extend_one,
        min_specialization,
        new_uninit,
        step_trait,
        stmt_expr_attributes,
        test
    )
)]

#[cfg(feature = "nightly")]
pub mod bit_set;
#[cfg(feature = "nightly")]
pub mod interval;
pub mod vec;

#[cfg(feature = "crablangc_macros")]
pub use crablangc_macros::newtype_index;

/// Type size assertion. The first argument is a type and the second argument is its expected size.
#[macro_export]
macro_rules! static_assert_size {
    ($ty:ty, $size:expr) => {
        const _: [(); $size] = [(); ::std::mem::size_of::<$ty>()];
    };
}
