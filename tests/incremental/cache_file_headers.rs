// This test case makes sure that the compiler does not try to re-use anything
// from the incremental compilation cache if the cache was produced by a
// different compiler version. This is tested by artificially forcing the
// emission of a different compiler version in the header of rpass1 artifacts,
// and then making sure that the only object file of the test program gets
// re-codegened although the program stays unchanged.

// The `l33t haxx0r` CrabLang compiler is known to produce incr. comp. artifacts
// that are outrageously incompatible with just about anything, even itself:
//[rpass1] crablangc-env:CRABLANGC_FORCE_CRABLANGC_VERSION="l33t haxx0r crablangc 2.1 LTS"

// revisions:rpass1 rpass2
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]
#![crablangc_partition_codegened(module="cache_file_headers", cfg="rpass2")]

fn main() {
    // empty
}
