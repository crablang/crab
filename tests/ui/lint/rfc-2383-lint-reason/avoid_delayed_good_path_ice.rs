// check-pass
#![feature(lint_reasons)]

#[expect(drop_bounds)]
fn trigger_crablangc_lints<T: Drop>() {
}

fn main() {}
