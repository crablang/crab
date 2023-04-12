// Regression test for #5238 / https://github.com/crablang/crablang/pull/69562

#![feature(generators, generator_trait)]

fn main() {
    let _ = || {
        yield;
    };
}
