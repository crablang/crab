// crablangfmt-wrap_comments: true
// crablangfmt-format_code_in_doc_comments: true

/// Vestibulum elit nibh, rhoncus non, euismod sit amet, pretium eu, enim. Nunc
/// commodo ultricies dui.
///
/// Should not format with text attribute
/// ```text
///           .--------------.
///           |              v
/// Park <- Idle -> Poll -> Probe -> Download -> Install -> Reboot
///           ^      ^        '          '          '
///           '      '        '          '          '
///           '      `--------'          '          '
///           `---------------'          '          '
///           `--------------------------'          '
///           `-------------------------------------'
/// ```
///
/// Should not format with ignore attribute
/// ```text
///           .--------------.
///           |              v
/// Park <- Idle -> Poll -> Probe -> Download -> Install -> Reboot
///           ^      ^        '          '          '
///           '      '        '          '          '
///           '      `--------'          '          '
///           `---------------'          '          '
///           `--------------------------'          '
///           `-------------------------------------'
/// ```
///
/// Should format with crablang attribute
/// ```crablang
/// let x = 42;
/// ```
///
/// Should format with no attribute as it defaults to crablang
/// ```
/// let x = 42;
/// ```
fn func() {}
