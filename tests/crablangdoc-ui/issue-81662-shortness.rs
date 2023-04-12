// compile-flags:--test --error-format=short
// normalize-stdout-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test "finished in \d+\.\d+s" -> "finished in $$TIME"
// failure-status: 101

/// ```crablang
/// foo();
/// ```
//~^^ ERROR cannot find function `foo` in this scope
fn foo() {
    println!("Hello, world!");
}
