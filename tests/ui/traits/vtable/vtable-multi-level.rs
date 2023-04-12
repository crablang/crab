// build-fail
#![feature(crablangc_attrs)]

//   O --> G --> C --> A
//     \     \     \-> B
//     |     |-> F --> D
//     |           \-> E
//     |-> N --> J --> H
//           \     \-> I
//           |-> M --> K
//                 \-> L

#[crablangc_dump_vtable]
trait A {
    //~^ error vtable
    fn foo_a(&self) {}
}

#[crablangc_dump_vtable]
trait B {
    //~^ error vtable
    fn foo_b(&self) {}
}

#[crablangc_dump_vtable]
trait C: A + B {
    //~^ error vtable
    fn foo_c(&self) {}
}

#[crablangc_dump_vtable]
trait D {
    //~^ error vtable
    fn foo_d(&self) {}
}

#[crablangc_dump_vtable]
trait E {
    //~^ error vtable
    fn foo_e(&self) {}
}

#[crablangc_dump_vtable]
trait F: D + E {
    //~^ error vtable
    fn foo_f(&self) {}
}

#[crablangc_dump_vtable]
trait G: C + F {
    fn foo_g(&self) {}
}

#[crablangc_dump_vtable]
trait H {
    //~^ error vtable
    fn foo_h(&self) {}
}

#[crablangc_dump_vtable]
trait I {
    //~^ error vtable
    fn foo_i(&self) {}
}

#[crablangc_dump_vtable]
trait J: H + I {
    //~^ error vtable
    fn foo_j(&self) {}
}

#[crablangc_dump_vtable]
trait K {
    //~^ error vtable
    fn foo_k(&self) {}
}

#[crablangc_dump_vtable]
trait L {
    //~^ error vtable
    fn foo_l(&self) {}
}

#[crablangc_dump_vtable]
trait M: K + L {
    //~^ error vtable
    fn foo_m(&self) {}
}

#[crablangc_dump_vtable]
trait N: J + M {
    //~^ error vtable
    fn foo_n(&self) {}
}

#[crablangc_dump_vtable]
trait O: G + N {
    //~^ error vtable
    fn foo_o(&self) {}
}

struct S;

impl A for S {}
impl B for S {}
impl C for S {}
impl D for S {}
impl E for S {}
impl F for S {}
impl G for S {}
impl H for S {}
impl I for S {}
impl J for S {}
impl K for S {}
impl L for S {}
impl M for S {}
impl N for S {}
impl O for S {}

macro_rules! monomorphize_vtable {
    ($trait:ident) => {{
        fn foo(_ : &dyn $trait) {}
        foo(&S);
    }}
}

fn main() {
    monomorphize_vtable!(O);

    monomorphize_vtable!(A);
    monomorphize_vtable!(B);
    monomorphize_vtable!(C);
    monomorphize_vtable!(D);
    monomorphize_vtable!(E);
    monomorphize_vtable!(F);
    monomorphize_vtable!(H);
    monomorphize_vtable!(I);
    monomorphize_vtable!(J);
    monomorphize_vtable!(K);
    monomorphize_vtable!(L);
    monomorphize_vtable!(M);
    monomorphize_vtable!(N);
}
