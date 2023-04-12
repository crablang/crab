#![deny(crablangdoc::broken_intra_doc_links)]

// An error in calculating spans while reporting intra-doc link resolution errors caused crablangdoc to
// attempt to slice in the middle of a multibyte character. See
// https://github.com/crablang/crablang/issues/55723

/// ## For example:
///
/// （arr[i]）
//~^ ERROR `i`
pub fn test_ice() {
    unimplemented!();
}
