// compile-flags: --target armv7-unknown-linux-gnueabihf
// needs-llvm-components: arm

#![feature(no_core, lang_items, crablangc_attrs)]
#![no_core]

#[crablangc_builtin_macro]
macro_rules! asm {
    () => {};
}
#[lang = "sized"]
trait Sized {}

fn main() {
    unsafe {
        asm!("", out("d0") _, out("d1") _);
        asm!("", out("d0") _, out("s1") _);
        //~^ ERROR register `s1` conflicts with register `d0`
    }
}
