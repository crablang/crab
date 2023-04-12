// unset-crablangc-env:CRABLANG_BACKTRACE
// compile-flags:-Z treat-err-as-bug=1
// error-pattern:stack backtrace:
// failure-status:101
// normalize-stderr-test "note: .*" -> ""
// normalize-stderr-test "thread 'crablangc' .*" -> ""
// normalize-stderr-test "  .*\n" -> ""

fn main() { missing_ident; }
