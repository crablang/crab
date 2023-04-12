#![allow(clippy::all)]

/// Test for https://github.com/crablang/crablang-clippy/issues/1588

fn main() {
    match 1 {
        1 => {},
        2 => {
            [0; 1];
        },
        _ => {},
    }
}
