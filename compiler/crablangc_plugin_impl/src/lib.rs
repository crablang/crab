//! Infrastructure for compiler plugins.
//!
//! Plugins are a deprecated way to extend the behavior of `crablangc` in various ways.
//!
//! See the [`plugin`
//! feature](https://doc.crablang.org/nightly/unstable-book/language-features/plugin.html)
//! of the Unstable Book for some examples.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_lint::LintStore;
use crablangc_macros::fluent_messages;

mod errors;
pub mod load;

fluent_messages! { "../messages.ftl" }

/// Structure used to register plugins.
///
/// A plugin registrar function takes an `&mut Registry` and should call
/// methods to register its plugins.
pub struct Registry<'a> {
    /// The `LintStore` allows plugins to register new lints.
    pub lint_store: &'a mut LintStore,
}
