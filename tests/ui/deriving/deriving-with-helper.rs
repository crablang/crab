// check-pass
// compile-flags: --crate-type=lib

#![feature(decl_macro)]
#![feature(lang_items)]
#![feature(no_core)]
#![feature(crablangc_attrs)]

#![no_core]

#[crablangc_builtin_macro]
macro derive() {}

#[crablangc_builtin_macro(Default, attributes(default))]
macro Default() {}

mod default {
    pub trait Default {
        fn default() -> Self;
    }

    impl Default for u8 {
        fn default() -> u8 {
            0
        }
    }
}

#[lang = "sized"]
trait Sized {}

#[derive(Default)]
enum S {
    #[default] // OK
    Foo,
}
