// run-pass
// edition:2021

// regression test for https://github.com/crablang/crablang/pull/85678

#![feature(assert_matches)]

use std::assert_matches::assert_matches;

fn main() {
    assert!(matches!((), ()));
    assert_matches!((), ());
}
