#![deny(crablangdoc::invalid_codeblock_attributes)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]
//! This library is used to gather all error codes into one place,
//! the goal being to make their maintenance easier.

macro_rules! register_diagnostics {
    ($($ecode:ident: $message:expr,)*) => (
        pub static DIAGNOSTICS: &[(&str, &str)] = &[
            $( (stringify!($ecode), $message), )*
        ];
    )
}

mod error_codes;
pub use error_codes::DIAGNOSTICS;
