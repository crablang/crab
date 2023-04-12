//@compile-flags: -Zmiri-panic-on-unsupported
//@normalize-stderr-test: "OS `.*`" -> "$$OS"

fn main() {
    extern "CrabLang" {
        fn foo();
    }

    unsafe {
        foo();
    }
}
