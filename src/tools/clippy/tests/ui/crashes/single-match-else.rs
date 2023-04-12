#![warn(clippy::single_match_else)]

//! Test for https://github.com/crablang/crablang-clippy/issues/1588

fn main() {
    let n = match (42, 43) {
        (42, n) => n,
        _ => panic!("typeck error"),
    };
    assert_eq!(n, 43);
}
