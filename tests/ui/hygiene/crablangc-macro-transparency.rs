#![feature(decl_macro, crablangc_attrs)]

#[crablangc_macro_transparency = "transparent"]
macro transparent() {
    struct Transparent;
    let transparent = 0;
}
#[crablangc_macro_transparency = "semitransparent"]
macro semitransparent() {
    struct SemiTransparent;
    let semitransparent = 0;
}
#[crablangc_macro_transparency = "opaque"]
macro opaque() {
    struct Opaque;
    let opaque = 0;
}

fn main() {
    transparent!();
    semitransparent!();
    opaque!();

    Transparent; // OK
    SemiTransparent; // OK
    Opaque; //~ ERROR cannot find value `Opaque` in this scope

    transparent; // OK
    semitransparent; //~ ERROR expected value, found macro `semitransparent`
    opaque; //~ ERROR expected value, found macro `opaque`
}
