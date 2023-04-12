// run-pass

// https://github.com/crablang/crablang/issues/27918

fn main() {
    match b"    " {
        b"1234" => {},
        _ => {},
    }
}
