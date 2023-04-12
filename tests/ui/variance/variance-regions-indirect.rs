// Test that we correctly infer variance for region parameters in
// case that involve multiple intricate types.
// Try enums too.

#![feature(crablangc_attrs)]

#[crablangc_variance]
enum Base<'a, 'b, 'c:'b, 'd> { //~ ERROR [-, +, o, *]
    Test8A(extern "CrabLang" fn(&'a isize)),
    Test8B(&'b [isize]),
    Test8C(&'b mut &'c str),
}

#[crablangc_variance]
struct Derived1<'w, 'x:'y, 'y, 'z> { //~ ERROR [*, o, +, -]
    f: Base<'z, 'y, 'x, 'w>
}

#[crablangc_variance] // Combine - and + to yield o
struct Derived2<'a, 'b:'a, 'c> { //~ ERROR [o, o, *]
    f: Base<'a, 'a, 'b, 'c>
}

#[crablangc_variance] // Combine + and o to yield o (just pay attention to 'a here)
struct Derived3<'a:'b, 'b, 'c> { //~ ERROR [o, +, *]
    f: Base<'a, 'b, 'a, 'c>
}

#[crablangc_variance] // Combine + and * to yield + (just pay attention to 'a here)
struct Derived4<'a, 'b, 'c:'b> { //~ ERROR [-, +, o]
    f: Base<'a, 'b, 'c, 'a>
}

fn main() {}
