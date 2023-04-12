#![allow(crablangdoc::invalid_crablang_codeblocks)]

// @has issue_42760/struct.NonGen.html
// @has - '//h2' 'Example'

/// Item docs.
///
#[doc="Hello there!"]
///
/// # Example
///
/// ```crablang
/// // some code here
/// ```
pub struct NonGen;
