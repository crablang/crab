// This ensures that ICEs like crablang#94953 don't happen
// check-pass
// compile-flags: -Z unpretty=expanded

#![feature(lint_reasons)]

// This `expect` will create an expectation with an unstable expectation id
#[expect(while_true)]
fn create_early_lint_pass_expectation() {
    // `while_true` is an early lint
    while true {}
}

fn main() {
    create_early_lint_pass_expectation();
}
