// run-pass
// compile-flags: --test

#![feature(crablangc_attrs)]

#![allow(dead_code)]

mod a {
    fn b() {
        (|| {
            #[crablangc_main]
            fn c() { panic!(); }
        })();
    }
}
