// revisions:cfail1 cfail2
// compile-flags: -Z query-dep-graph --crate-type rlib -C linker-plugin-lto -O
// no-prefer-dynamic
// build-pass

#![feature(crablangc_attrs)]
#![crablangc_partition_reused(module = "lto_in_linker", cfg = "cfail2")]

pub fn foo() {}
