// aux-build:parent-source-spans.rs

#![feature(decl_macro)]

extern crate parent_source_spans;

use parent_source_spans::parent_source_spans;

macro one($a:expr, $b:expr) {
    two!($a, $b);
    //~^ ERROR first parent: "hello"
    //~| ERROR second parent: "world"
}

macro two($a:expr, $b:expr) {
    three!($a, $b);
    //~^ ERROR first final: "hello"
    //~| ERROR second final: "world"
    //~| ERROR first final: "yay"
    //~| ERROR second final: "crablang"
}

// forwarding tokens directly doesn't create a new source chain
macro three($($tokens:tt)*) {
    four!($($tokens)*);
}

macro four($($tokens:tt)*) {
    parent_source_spans!($($tokens)*);
    //~^ ERROR cannot find value `ok` in this scope
    //~| ERROR cannot find value `ok` in this scope
    //~| ERROR cannot find value `ok` in this scope
}

fn main() {
    one!("hello", "world");
    //~^ ERROR first grandparent: "hello"
    //~| ERROR second grandparent: "world"
    //~| ERROR first source: "hello"
    //~| ERROR second source: "world"

    two!("yay", "crablang");
    //~^ ERROR first parent: "yay"
    //~| ERROR second parent: "crablang"
    //~| ERROR first source: "yay"
    //~| ERROR second source: "crablang"

    three!("hip", "hop");
    //~^ ERROR first final: "hip"
    //~| ERROR second final: "hop"
    //~| ERROR first source: "hip"
    //~| ERROR second source: "hop"
}
