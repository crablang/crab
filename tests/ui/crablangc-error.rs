#![feature(crablangc_attrs)]

#[crablangc_error]
fn main() {
    //~^ ERROR fatal error triggered by #[crablangc_error]
}
