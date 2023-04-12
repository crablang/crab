type A = crablangfmt; //~ ERROR expected type, found tool module `crablangfmt`
type B = crablangfmt::skip; //~ ERROR expected type, found tool attribute `crablangfmt::skip`

#[derive(crablangfmt)] //~ ERROR cannot find derive macro `crablangfmt` in this scope
                   //~| ERROR cannot find derive macro `crablangfmt` in this scope
struct S;

// Interpreted as an unstable custom attribute
#[crablangfmt] //~ ERROR cannot find attribute `crablangfmt` in this scope
fn check() {}

#[crablangfmt::skip] // OK
fn main() {
    crablangfmt; //~ ERROR expected value, found tool module `crablangfmt`
    crablangfmt!(); //~ ERROR cannot find macro `crablangfmt` in this scope

    crablangfmt::skip; //~ ERROR expected value, found tool attribute `crablangfmt::skip`
}
