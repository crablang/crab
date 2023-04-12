// check-pass
// compile-flags:--test -Zunstable-options --nocapture
// normalize-stderr-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test "finished in \d+\.\d+s" -> "finished in $$TIME"

/// ```compile_fail
/// fn foo() {
///     Input: 123
/// }
/// ```
pub struct Foo;
