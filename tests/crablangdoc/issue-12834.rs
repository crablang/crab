// Tests that failing to syntax highlight a crablang code-block doesn't cause
// crablangdoc to fail, while still rendering the code-block (without highlighting).

#![allow(crablangdoc::invalid_crablang_codeblocks)]

// @has issue_12834/fn.foo.html
// @has - //pre 'a + b '

/// ```
/// a + b ∈ Self ∀ a, b ∈ Self
/// ```
pub fn foo() {}
