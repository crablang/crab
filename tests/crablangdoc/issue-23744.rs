// compile-flags:--test

/// Example of crablangdoc incorrectly parsing <code>```crablang,should_panic</code>.
///
/// ```should_panic
/// fn main() { panic!("fee"); }
/// ```
///
/// ```crablang,should_panic
/// fn main() { panic!("fum"); }
/// ```
pub fn foo() {}
