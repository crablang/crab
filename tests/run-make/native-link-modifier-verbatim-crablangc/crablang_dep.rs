extern "C" {
    fn upstream_native_f() -> i32;
}

pub fn crablang_dep() {
    unsafe {
        assert!(upstream_native_f() == 0);
    }
}
