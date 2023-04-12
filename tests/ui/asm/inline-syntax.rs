// revisions: x86_64 arm
//[x86_64] compile-flags: --target x86_64-unknown-linux-gnu
//[x86_64] check-pass
//[x86_64] needs-llvm-components: x86
//[x86_64_allowed] compile-flags: --target x86_64-unknown-linux-gnu
//[x86_64_allowed] check-pass
//[x86_64_allowed] needs-llvm-components: x86
//[arm] compile-flags: --target armv7-unknown-linux-gnueabihf
//[arm] build-fail
//[arm] needs-llvm-components: arm
// needs-asm-support

#![feature(no_core, lang_items, crablangc_attrs)]
#![crate_type = "rlib"]
#![no_core]
#![cfg_attr(x86_64_allowed, allow(bad_asm_style))]

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

pub fn main() {
    unsafe {
        asm!(".intel_syntax noprefix", "nop");
        //[x86_64]~^ WARN avoid using `.intel_syntax`
        //[arm]~^^ ERROR unknown directive
        asm!(".intel_syntax aaa noprefix", "nop");
        //[x86_64]~^ WARN avoid using `.intel_syntax`
        //[arm]~^^ ERROR unknown directive
        asm!(".att_syntax noprefix", "nop");
        //[x86_64]~^ WARN avoid using `.att_syntax`
        //[arm]~^^ ERROR unknown directive
        asm!(".att_syntax bbb noprefix", "nop");
        //[x86_64]~^ WARN avoid using `.att_syntax`
        //[arm]~^^ ERROR unknown directive
        asm!(".intel_syntax noprefix; nop");
        //[x86_64]~^ WARN avoid using `.intel_syntax`
        //[arm]~^^ ERROR unknown directive

        asm!(
            r"
            .intel_syntax noprefix
            nop"
        );
        //[x86_64]~^^^ WARN avoid using `.intel_syntax`
        //[arm]~^^^^ ERROR unknown directive
    }
}

global_asm!(".intel_syntax noprefix", "nop");
//[x86_64]~^ WARN avoid using `.intel_syntax`
// Assembler errors don't have line numbers, so no error on ARM
