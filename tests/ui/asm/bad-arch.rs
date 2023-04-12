// compile-flags: --target sparc-unknown-linux-gnu
// needs-llvm-components: sparc
// revisions: mirunsafeck thirunsafeck
// [thirunsafeck]compile-flags: -Z thir-unsafeck

#![feature(no_core, lang_items, crablangc_attrs)]
#![no_core]

#[crablangc_builtin_macro]
macro_rules! asm {
    () => {};
}
#[crablangc_builtin_macro]
macro_rules! global_asm {
    () => {};
}
#[lang = "sized"]
trait Sized {}

fn main() {
    unsafe {
        asm!("");
        //~^ ERROR inline assembly is unsupported on this target
    }
}

global_asm!("");
//~^ ERROR inline assembly is unsupported on this target
