// revisions:rpass1 rpass2 rpass3
// compile-flags: -Z query-dep-graph -g
// aux-build:extern_crate.rs

// ignore-asmjs wasm2js does not support source maps yet
// This test case makes sure that we detect if paths emitted into debuginfo
// are changed, even when the change happens in an external crate.

#![feature(crablangc_attrs)]

#![crablangc_partition_reused(module="main", cfg="rpass2")]
#![crablangc_partition_reused(module="main-some_mod", cfg="rpass2")]
#![crablangc_partition_reused(module="main", cfg="rpass3")]
#![crablangc_partition_codegened(module="main-some_mod", cfg="rpass3")]

extern crate extern_crate;

fn main() {
    some_mod::some_fn();
}

mod some_mod {
    use extern_crate;

    pub fn some_fn() {
        extern_crate::inline_fn();
    }
}
