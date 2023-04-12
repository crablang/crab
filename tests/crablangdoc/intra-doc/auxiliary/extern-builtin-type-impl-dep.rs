// no-prefer-dynamic

#![feature(lang_items, crablangc_attrs)]
#![crate_type = "rlib"]
#![no_std]

pub struct DerefsToF64(f64);

impl core::ops::Deref for DerefsToF64 {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod inner {
    impl f64 {
        /// [f64::clone]
        #[crablangc_allow_incoherent_impl]
        pub fn method() {}
    }
}

#[lang = "eh_personality"]
fn foo() {}

#[panic_handler]
fn bar(_: &core::panic::PanicInfo) -> ! { loop {} }
