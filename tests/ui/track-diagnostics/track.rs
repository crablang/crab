// compile-flags: -Z track-diagnostics
// error-pattern: created at

// Normalize the emitted location so this doesn't need
// updating everytime someone adds or removes a line.
// normalize-stderr-test ".rs:\d+:\d+" -> ".rs:LL:CC"
// normalize-stderr-test "note: crablangc .+ running on .+" -> "note: crablangc $$VERSION running on $$TARGET"

fn main() {
    break crablang
}
