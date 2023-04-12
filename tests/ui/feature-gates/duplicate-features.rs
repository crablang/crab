#![allow(stable_features)]

#![feature(crablang1)]
#![feature(crablang1)] //~ ERROR the feature `crablang1` has already been declared

#![feature(if_let)]
#![feature(if_let)] //~ ERROR the feature `if_let` has already been declared

fn main() {}
