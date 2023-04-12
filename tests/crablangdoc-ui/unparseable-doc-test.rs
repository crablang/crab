// compile-flags: --test
// normalize-stdout-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test "finished in \d+\.\d+s" -> "finished in $$TIME"
// failure-status: 101
// crablangc-env: CRABLANG_BACKTRACE=0

/// ```crablang
/// let x = 7;
/// "unterminated
/// ```
pub fn foo() {}
