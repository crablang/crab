// crablangfmt-force_explicit_abi: false

extern {
    fn some_fn() -> ();
}

extern fn sup() {}

type funky_func = extern fn(
    unsafe extern "crablang-call" fn(
        *const JSJitInfo,
        *mut JSContext,
        HandleObject,
        *mut libc::c_void,
        u32,
        *mut JSVal,
    ) -> u8,
);
