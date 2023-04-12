// run-pass

// https://github.com/crablang/crablang/issues/25574

const A: [u8; 4] = *b"fooo";

fn main() {
    match *b"xxxx" {
        A => {},
        _ => {}
    }
}
