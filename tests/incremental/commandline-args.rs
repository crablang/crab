// Test that changing a tracked commandline argument invalidates
// the cache while changing an untracked one doesn't.

// ignore-asmjs wasm2js does not support source maps yet
// revisions:rpass1 rpass2 rpass3 rpass4
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]

#![crablangc_partition_codegened(module="commandline_args", cfg="rpass2")]
#![crablangc_partition_reused(module="commandline_args", cfg="rpass3")]
#![crablangc_partition_codegened(module="commandline_args", cfg="rpass4")]

// Between revisions 1 and 2, we are changing the debuginfo-level, which should
// invalidate the cache. Between revisions 2 and 3, we are adding `--verbose`
// which should have no effect on the cache. Between revisions, we are adding
// `--remap-path-prefix` which should invalidate the cache:
//[rpass1] compile-flags: -C debuginfo=0
//[rpass2] compile-flags: -C debuginfo=2
//[rpass3] compile-flags: -C debuginfo=2 --verbose
//[rpass4] compile-flags: -C debuginfo=2 --verbose --remap-path-prefix=/home/bors/crablang=src

pub fn main() {
    // empty
}
