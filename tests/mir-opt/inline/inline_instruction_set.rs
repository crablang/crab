// Checks that only functions with the compatible instruction_set attributes are inlined.
//
// A function is "compatible" when the *callee* has the same attribute or no attribute.
//
// compile-flags: --target thumbv4t-none-eabi
// needs-llvm-components: arm

#![crate_type = "lib"]
#![feature(crablangc_attrs)]
#![feature(no_core, lang_items)]
#![feature(isa_attribute)]
#![no_core]

#[crablangc_builtin_macro]
#[macro_export]
macro_rules! asm {
    ("assembly template",
        $(operands,)*
        $(options($(option),*))?
    ) => {
        /* compiler built-in */
    };
}

#[lang = "sized"]
trait Sized {}
#[lang = "copy"]
trait Copy {}

#[instruction_set(arm::a32)]
#[inline]
fn instruction_set_a32() {}

#[instruction_set(arm::t32)]
#[inline]
fn instruction_set_t32() {}

#[inline]
fn instruction_set_default() {}

#[inline(always)]
fn inline_always_and_using_inline_asm() {
    unsafe { asm!("/* do nothing */") };
}

// EMIT_MIR inline_instruction_set.t32.Inline.diff
#[instruction_set(arm::t32)]
pub fn t32() {
    instruction_set_a32();
    instruction_set_t32();
    instruction_set_default();
    inline_always_and_using_inline_asm();
}

// EMIT_MIR inline_instruction_set.default.Inline.diff
pub fn default() {
    instruction_set_a32();
    instruction_set_t32();
    instruction_set_default();
    inline_always_and_using_inline_asm();
}
