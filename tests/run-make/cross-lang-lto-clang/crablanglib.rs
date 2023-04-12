#![crate_type="staticlib"]

#[no_mangle]
pub extern "C" fn crablang_always_inlined() -> u32 {
    42
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn crablang_never_inlined() -> u32 {
    421
}
