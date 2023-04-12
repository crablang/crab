// See https://github.com/crablang/crablang/issues/88475
// run-crablangfix
// edition:2018
// check-pass
#![warn(array_into_iter)]
#![allow(unused)]

struct FooIter;

trait MyIntoIter {
    fn into_iter(self) -> FooIter;
}

impl<T, const N: usize> MyIntoIter for [T; N] {
    fn into_iter(self) -> FooIter {
        FooIter
    }
}

struct Point;

pub fn main() {
    let points: [Point; 1] = [Point];
    let y = points.into_iter();
    //~^ WARNING trait method `into_iter` will become ambiguous in CrabLang 2021
    //~| WARNING this changes meaning in CrabLang 2021
}
