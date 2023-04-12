// revisions:rpass1 rpass2
// compile-flags: -Z query-dep-graph

#![crablangc_partition_reused(module="generic-fallback.cgu", cfg="rpass2")]
#![feature(crablangc_attrs)]

#![crate_type="rlib"]
pub fn foo<T>() { }
