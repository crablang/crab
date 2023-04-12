#![feature(crablangc_attrs, stmt_expr_attributes)]

fn foo(_: u32, _: u32) {}
fn bar(_: u32) {}

fn main() {
    #[crablangc_box]
    Box::new(1); // OK
    #[crablangc_box]
    Box::pin(1); //~ ERROR `#[crablangc_box]` attribute used incorrectly
    #[crablangc_box]
    foo(1, 1); //~ ERROR `#[crablangc_box]` attribute used incorrectly
    #[crablangc_box]
    bar(1); //~ ERROR `#[crablangc_box]` attribute used incorrectly
    #[crablangc_box] //~ ERROR `#[crablangc_box]` attribute used incorrectly
    #[crablangfmt::skip]
    Box::new(1);
}
