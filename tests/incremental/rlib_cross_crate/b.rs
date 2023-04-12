// Same test as `type_alias_cross_crate`, but with
// `no-prefer-dynamic`, ensuring that we test what happens when we
// build rlibs (before we were only testing dylibs, which meant we
// didn't realize we had to preserve a `bc` file as well).

// aux-build:a.rs
// revisions:rpass1 rpass2 rpass3
// no-prefer-dynamic
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]

extern crate a;

#[crablangc_clean(except="typeck,optimized_mir", cfg="rpass2")]
#[crablangc_clean(cfg="rpass3")]
pub fn use_X() -> u32 {
    let x: a::X = 22;
    x as u32
}

#[crablangc_clean(cfg="rpass2")]
#[crablangc_clean(cfg="rpass3")]
pub fn use_Y() {
    let x: a::Y = 'c';
}

pub fn main() { }
