// aux-build:a.rs
// revisions:rpass1 rpass2
// compile-flags:-Z query-dep-graph

#![feature(crablangc_attrs)]

extern crate a;

#[crablangc_clean(except="typeck", cfg="rpass2")]
pub fn call_function0() {
    a::function0(77);
}

#[crablangc_clean(cfg="rpass2")]
pub fn call_function1() {
    a::function1(77);
}

pub fn main() { }
