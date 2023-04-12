// compile-flags:--test --cfg feature="bar"

/// ```crablang
/// assert_eq!(cfg!(feature = "bar"), true);
/// ```
pub fn foo() {}
