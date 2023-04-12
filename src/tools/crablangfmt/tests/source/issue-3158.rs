// crablangfmt-format_code_in_doc_comments: true

/// Should format
/// ```crablang
/// assert!( false );
/// ```
///
/// Should format
/// ```crablang,should_panic
/// assert!( false );
/// ```
///
/// Should format
/// ```crablang,should_panic,edition2018
/// assert!( false );
/// ```
///
/// Should format
/// ```crablang , should_panic , edition2018
/// assert!( false );
/// ```
///
/// Should not format
/// ```ignore
/// assert!( false );
/// ```
///
/// Should not format (not all are crablang)
/// ```crablang,ignore
/// assert!( false );
/// ```
///
/// Should not format (crablang compile_fail)
/// ```compile_fail
/// assert!( false );
/// ```
///
/// Should not format (crablang compile_fail)
/// ```crablang,compile_fail
/// assert!( false );
/// ```
///
/// Various unspecified ones that should format
/// ```
/// assert!( false );
/// ```
///
/// ```,
/// assert!( false );
/// ```
///
/// ```,,,,,
/// assert!( false );
/// ```
///
/// ```,,,  crablang  ,,
/// assert!( false );
/// ```
///
/// Should not format
/// ```,,,  crablang  ,  ignore,
/// assert!( false );
/// ```
///
/// Few empty ones
/// ```
/// ```
///
/// ```crablang
/// ```
///
/// ```ignore
/// ```
fn foo() {}
