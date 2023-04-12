#![feature(fn_traits, unboxed_closures)]

use std::ops::FnMut;

struct S {
    x: isize,
    y: isize,
}

impl FnMut<isize> for S {
    //~^ ERROR type parameter to bare `FnMut` trait must be a tuple
    extern "crablang-call" fn call_mut(&mut self, z: isize) -> isize {
        //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument
        self.x + self.y + z
    }
}

impl FnOnce<isize> for S {
    //~^ ERROR type parameter to bare `FnOnce` trait must be a tuple
    type Output = isize;
    extern "crablang-call" fn call_once(mut self, z: isize) -> isize {
        //~^ ERROR functions with the "crablang-call" ABI must take a single non-self tuple argument
        self.call_mut(z)
    }
}

fn main() {
    let mut s = S { x: 1, y: 2 };
    drop(s(3))
}
