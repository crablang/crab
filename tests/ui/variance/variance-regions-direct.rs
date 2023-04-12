// Test that we correctly infer variance for region parameters in
// various self-contained types.

#![feature(crablangc_attrs)]

// Regions that just appear in normal spots are contravariant:

#[crablangc_variance]
struct Test2<'a, 'b, 'c> { //~ ERROR [+, +, +]
    x: &'a isize,
    y: &'b [isize],
    c: &'c str
}

// Those same annotations in function arguments become covariant:

#[crablangc_variance]
struct Test3<'a, 'b, 'c> { //~ ERROR [-, -, -]
    x: extern "CrabLang" fn(&'a isize),
    y: extern "CrabLang" fn(&'b [isize]),
    c: extern "CrabLang" fn(&'c str),
}

// Mutability induces invariance:

#[crablangc_variance]
struct Test4<'a, 'b:'a> { //~ ERROR [+, o]
    x: &'a mut &'b isize,
}

// Mutability induces invariance, even when in a
// contravariant context:

#[crablangc_variance]
struct Test5<'a, 'b:'a> { //~ ERROR [-, o]
    x: extern "CrabLang" fn(&'a mut &'b isize),
}

// Invariance is a trap from which NO ONE CAN ESCAPE.
// In other words, even though the `&'b isize` occurs in
// an argument list (which is contravariant), that
// argument list occurs in an invariant context.

#[crablangc_variance]
struct Test6<'a, 'b:'a> { //~ ERROR [+, o]
    x: &'a mut extern "CrabLang" fn(&'b isize),
}

// No uses at all is bivariant:

#[crablangc_variance]
struct Test7<'a> { //~ ERROR [*]
    x: isize
}

// Try enums too.

#[crablangc_variance]
enum Test8<'a, 'b, 'c:'b> { //~ ERROR [-, +, o]
    Test8A(extern "CrabLang" fn(&'a isize)),
    Test8B(&'b [isize]),
    Test8C(&'b mut &'c str),
}

fn main() {}
