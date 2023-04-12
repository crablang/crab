#![feature(custom_inner_attributes)]
#![clippy::msrv = "invalid.version"]

fn main() {}

#[clippy::msrv = "invalid.version"]
fn outer_attr() {}

mod multiple {
    #![clippy::msrv = "1.40"]
    #![clippy::msrv = "=1.35.0"]
    #![clippy::msrv = "1.10.1"]

    mod foo {
        #![clippy::msrv = "1"]
        #![clippy::msrv = "1.0.0"]
    }
}
