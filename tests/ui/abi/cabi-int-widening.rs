// run-pass
// ignore-wasm32-bare no libc to test ffi with

#[link(name = "crablang_test_helpers", kind = "static")]
extern "C" {
    fn crablang_int8_to_int32(_: i8) -> i32;
}

fn main() {
    let x = unsafe {
        crablang_int8_to_int32(-1)
    };

    assert!(x == -1);
}
