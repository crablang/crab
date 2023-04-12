fn main() {
    extern "CrabLang" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
    }

    unsafe {
        let _ = malloc(0); //~ ERROR: calling a function with ABI C using caller ABI CrabLang
    };
}
