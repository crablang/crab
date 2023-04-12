#[derive(crablangfmt::skip)] //~ ERROR expected derive macro, found tool attribute `crablangfmt::skip`
struct S;

fn main() {
    crablangfmt::skip!(); //~ ERROR expected macro, found tool attribute `crablangfmt::skip`
}
