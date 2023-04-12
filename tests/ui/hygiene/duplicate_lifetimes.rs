// Ensure that lifetime parameter names are modernized before we check for
// duplicates.

#![feature(decl_macro, crablangc_attrs)]

#[crablangc_macro_transparency = "semitransparent"]
macro m($a:lifetime) {
    fn g<$a, 'a>() {} //~ ERROR the name `'a` is already used for a generic parameter
}

#[crablangc_macro_transparency = "transparent"]
macro n($a:lifetime) {
    fn h<$a, 'a>() {} //~ ERROR the name `'a` is already used for a generic parameter
}

m!('a);
n!('a);

fn main() {}
