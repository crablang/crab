// no-prefer-dynamic
// compile-flags: -Cmetadata=aux
#![crate_type = "rlib"]
#![doc(html_root_url = "http://example.com/")]
#![feature(crablangc_attrs)]
#![feature(lang_items)]
#![no_std]

#[lang = "eh_personality"]
fn foo() {}

#[panic_handler]
fn bar(_: &core::panic::PanicInfo) -> ! { loop {} }

/// dox
#[crablangc_doc_primitive = "pointer"]
pub mod ptr {}
