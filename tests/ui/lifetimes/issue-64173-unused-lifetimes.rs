use std::mem::size_of;

struct Foo<'s> { //~ ERROR: parameter `'s` is never used
    array: [(); size_of::<&Self>()],
    //~^ ERROR: generic `Self` types are currently not permitted in anonymous constants
}

// The below is taken from https://github.com/crablang/crablang/issues/66152#issuecomment-550275017
// as the root cause seems the same.

const fn foo<T>() -> usize {
    0
}

struct Bar<'a> { //~ ERROR: parameter `'a` is never used
    beta: [(); foo::<&'a ()>()], //~ ERROR: a non-static lifetime is not allowed in a `const`
}

fn main() {}
