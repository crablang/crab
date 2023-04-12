// This issue reproduces an ICE on compile
// Fails on 2020-02-08 nightly
// regressed commit: https://github.com/crablang/crablang/commit/f8fd4624474a68bd26694eff3536b9f3a127b2d3
//
// check-pass

#![feature(generator_trait)]
#![feature(generators)]

use std::ops::Generator;

fn gen() -> impl Generator<usize> {
    |_: usize| {
        println!("-> {}", yield);
    }
}

fn main() {}
