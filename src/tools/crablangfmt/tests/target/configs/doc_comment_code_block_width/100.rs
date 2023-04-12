// crablangfmt-format_code_in_doc_comments: true
// crablangfmt-doc_comment_code_block_width: 100

/// ```crablang
/// impl Test {
///     pub const fn from_bytes(v: &[u8]) -> Result<Self, ParserError> {
///         Self::from_bytes_manual_slice(v, 0, v.len())
///     }
/// }
/// ```

impl Test {
    pub const fn from_bytes(v: &[u8]) -> Result<Self, ParserError> {
        Self::from_bytes_manual_slice(v, 0, v.len())
    }
}
