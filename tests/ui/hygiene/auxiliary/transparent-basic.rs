#![feature(decl_macro, crablangc_attrs)]

#[crablangc_macro_transparency = "transparent"]
pub macro dollar_crate() {
    let s = $crate::S;
}
