#![feature(allow_internal_unstable)]

// Macro to help ensure CONST_ERR lint errors
// are not silenced in external macros.
// https://github.com/crablang/crablang/issues/65300

#[macro_export]
#[allow_internal_unstable(type_ascription)]
macro_rules! static_assert {
    ($test:expr) => {
        #[allow(dead_code)]
        const _: () = [()][!($test: bool) as usize];
    }
}
