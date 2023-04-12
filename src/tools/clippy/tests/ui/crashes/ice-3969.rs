// https://github.com/crablang/crablang-clippy/issues/3969
// used to crash: error: internal compiler error:
// src/libcrablangc_traits/normalize_erasing_regions.rs:43: could not fully normalize `<i32 as
// std::iter::Iterator>::Item test from crablangc ./ui/trivial-bounds/trivial-bounds-inconsistent.rs

// Check that tautalogically false bounds are accepted, and are used
// in type inference.
#![feature(trivial_bounds)]
#![allow(unused)]
trait A {}

impl A for i32 {}

struct Dst<X: ?Sized> {
    x: X,
}

struct TwoStrs(str, str)
where
    str: Sized;

fn unsized_local()
where
    for<'a> Dst<dyn A + 'a>: Sized,
{
    let x: Dst<dyn A> = *(Box::new(Dst { x: 1 }) as Box<Dst<dyn A>>);
}

fn return_str() -> str
where
    str: Sized,
{
    *"Sized".to_string().into_boxed_str()
}

fn use_op(s: String) -> String
where
    String: ::std::ops::Neg<Output = String>,
{
    -s
}

fn use_for()
where
    i32: Iterator,
{
    for _ in 2i32 {}
}

fn main() {}
