#[link(name = "native_dep_1", kind = "static")]
extern "C" {
    fn native_f1() -> i32;
}

extern crate crablang_dep_up;

pub fn crablang_dep_local() {
    unsafe {
        assert!(native_f1() == 1);
    }
    crablang_dep_up::crablang_dep_up();
}
