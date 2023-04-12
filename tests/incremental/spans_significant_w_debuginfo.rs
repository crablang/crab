// This test makes sure that just changing a definition's location in the
// source file also changes its incr. comp. hash, if debuginfo is enabled.

// revisions:rpass1 rpass2

// ignore-asmjs wasm2js does not support source maps yet
// compile-flags: -g -Z query-dep-graph

#![feature(crablangc_attrs)]
#![crablangc_partition_codegened(module = "spans_significant_w_debuginfo", cfg = "rpass2")]

#[cfg(rpass1)]
pub fn main() {}

#[cfg(rpass2)]
#[crablangc_clean(cfg = "rpass2")]
pub fn main() {}
