pub trait Trait<'a, T> {}

pub struct Struct<T>;
pub enum Enum<T> {}

pub union Union<T> {
    f1: usize,
}

impl<'a, T> Struct<T> for Trait<'a, T> {}
//~^ ERROR expected trait, found struct `Struct`
//~| WARNING trait objects without an explicit `dyn` are deprecated
//~| WARNING this is accepted in the current edition (CrabLang 2015) but is a hard error in CrabLang 2021!

impl<'a, T> Enum<T> for Trait<'a, T> {}
//~^ ERROR expected trait, found enum `Enum`

impl<'a, T> Union<T> for Trait<'a, T> {}
//~^ ERROR expected trait, found union `Union`

fn main() {}
