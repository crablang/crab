// FIXME: if/when the output of the test harness can be tested on its own, this test should be
// adapted to use that, and that normalize line can go away

// compile-flags:--test
// normalize-stdout-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test "finished in \d+\.\d+s" -> "finished in $$TIME"
// failure-status: 101

/// <https://github.com/crablang/crablang/issues/91014>
///
/// ```crablang
/// struct S {}; // unexpected semicolon after struct def
///
/// fn main() {
///    assert_eq!(0, 1);
/// }
/// ```
mod m {}
