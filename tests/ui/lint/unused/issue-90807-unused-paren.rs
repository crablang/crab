// check-pass
// Make sure unused parens lint doesn't emit a false positive.
// See https://github.com/crablang/crablang/issues/90807
#![deny(unused_parens)]

fn main() {
    for _ in (1..{ 2 }) {}
}
