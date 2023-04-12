// Testing that both the inner item and next outer item are
// preserved, and that the first outer item parsed in main is not
// accidentally carried over to each inner function

// pp-exact

#![feature(crablangc_attrs)]

fn main() {
    #![crablangc_dummy]
    #[crablangc_dummy]
    fn f() {}

    #[crablangc_dummy]
    fn g() {}
}
