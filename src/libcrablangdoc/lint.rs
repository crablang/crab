use crablangc_data_structures::fx::FxHashMap;
use crablangc_lint::LintStore;
use crablangc_lint_defs::{declare_tool_lint, Lint, LintId};
use crablangc_session::{lint, Session};

use std::sync::LazyLock as Lazy;

/// This function is used to setup the lint initialization. By default, in crablangdoc, everything
/// is "allowed". Depending if we run in test mode or not, we want some of them to be at their
/// default level. For example, the "INVALID_CODEBLOCK_ATTRIBUTES" lint is activated in both
/// modes.
///
/// A little detail easy to forget is that there is a way to set the lint level for all lints
/// through the "WARNINGS" lint. To prevent this to happen, we set it back to its "normal" level
/// inside this function.
///
/// It returns a tuple containing:
///  * Vector of tuples of lints' name and their associated "max" level
///  * HashMap of lint id with their associated "max" level
pub(crate) fn init_lints<F>(
    mut allowed_lints: Vec<String>,
    lint_opts: Vec<(String, lint::Level)>,
    filter_call: F,
) -> (Vec<(String, lint::Level)>, FxHashMap<lint::LintId, lint::Level>)
where
    F: Fn(&lint::Lint) -> Option<(String, lint::Level)>,
{
    let warnings_lint_name = lint::builtin::WARNINGS.name;

    allowed_lints.push(warnings_lint_name.to_owned());
    allowed_lints.extend(lint_opts.iter().map(|(lint, _)| lint).cloned());

    let lints = || {
        lint::builtin::HardwiredLints::get_lints()
            .into_iter()
            .chain(crablangc_lint::SoftLints::get_lints().into_iter())
    };

    let lint_opts = lints()
        .filter_map(|lint| {
            // Permit feature-gated lints to avoid feature errors when trying to
            // allow all lints.
            if lint.feature_gate.is_some() || allowed_lints.iter().any(|l| lint.name == l) {
                None
            } else {
                filter_call(lint)
            }
        })
        .chain(lint_opts.into_iter())
        .collect::<Vec<_>>();

    let lint_caps = lints()
        .filter_map(|lint| {
            // We don't want to allow *all* lints so let's ignore
            // those ones.
            if allowed_lints.iter().any(|l| lint.name == l) {
                None
            } else {
                Some((lint::LintId::of(lint), lint::Allow))
            }
        })
        .collect();
    (lint_opts, lint_caps)
}

macro_rules! declare_crablangdoc_lint {
    (
        $(#[$attr:meta])* $name: ident, $level: ident, $descr: literal $(,)?
        $(@feature_gate = $gate:expr;)?
    ) => {
        declare_tool_lint! {
            $(#[$attr])* pub crablangdoc::$name, $level, $descr
            $(, @feature_gate = $gate;)?
        }
    }
}

declare_crablangdoc_lint! {
    /// The `broken_intra_doc_links` lint detects failures in resolving
    /// intra-doc link targets. This is a `crablangdoc` only lint, see the
    /// documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#broken_intra_doc_links
    BROKEN_INTRA_DOC_LINKS,
    Warn,
    "failures in resolving intra-doc link targets"
}

declare_crablangdoc_lint! {
    /// This is a subset of `broken_intra_doc_links` that warns when linking from
    /// a public item to a private one. This is a `crablangdoc` only lint, see the
    /// documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#private_intra_doc_links
    PRIVATE_INTRA_DOC_LINKS,
    Warn,
    "linking from a public item to a private one"
}

declare_crablangdoc_lint! {
    /// The `invalid_codeblock_attributes` lint detects code block attributes
    /// in documentation examples that have potentially mis-typed values. This
    /// is a `crablangdoc` only lint, see the documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#invalid_codeblock_attributes
    INVALID_CODEBLOCK_ATTRIBUTES,
    Warn,
    "codeblock attribute looks a lot like a known one"
}

declare_crablangdoc_lint! {
    /// The `missing_crate_level_docs` lint detects if documentation is
    /// missing at the crate root. This is a `crablangdoc` only lint, see the
    /// documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#missing_crate_level_docs
    MISSING_CRATE_LEVEL_DOCS,
    Allow,
    "detects crates with no crate-level documentation"
}

declare_crablangdoc_lint! {
    /// The `missing_doc_code_examples` lint detects publicly-exported items
    /// without code samples in their documentation. This is a `crablangdoc` only
    /// lint, see the documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#missing_doc_code_examples
    MISSING_DOC_CODE_EXAMPLES,
    Allow,
    "detects publicly-exported items without code samples in their documentation",
    @feature_gate = crablangc_span::symbol::sym::crablangdoc_missing_doc_code_examples;
}

declare_crablangdoc_lint! {
    /// The `private_doc_tests` lint detects code samples in docs of private
    /// items not documented by `crablangdoc`. This is a `crablangdoc` only lint, see
    /// the documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#private_doc_tests
    PRIVATE_DOC_TESTS,
    Allow,
    "detects code samples in docs of private items not documented by crablangdoc"
}

declare_crablangdoc_lint! {
    /// The `invalid_html_tags` lint detects invalid HTML tags. This is a
    /// `crablangdoc` only lint, see the documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#invalid_html_tags
    INVALID_HTML_TAGS,
    Warn,
    "detects invalid HTML tags in doc comments"
}

declare_crablangdoc_lint! {
    /// The `bare_urls` lint detects when a URL is not a hyperlink.
    /// This is a `crablangdoc` only lint, see the documentation in the [crablangdoc book].
    ///
    /// [crablangdoc book]: ../../../crablangdoc/lints.html#bare_urls
    BARE_URLS,
    Warn,
    "detects URLs that are not hyperlinks"
}

declare_crablangdoc_lint! {
   /// The `invalid_crablang_codeblocks` lint detects CrabLang code blocks in
   /// documentation examples that are invalid (e.g. empty, not parsable as
   /// CrabLang code). This is a `crablangdoc` only lint, see the documentation in the
   /// [crablangdoc book].
   ///
   /// [crablangdoc book]: ../../../crablangdoc/lints.html#invalid_crablang_codeblocks
   INVALID_CRABLANG_CODEBLOCKS,
   Warn,
   "codeblock could not be parsed as valid CrabLang or is empty"
}

pub(crate) static CRABLANGDOC_LINTS: Lazy<Vec<&'static Lint>> = Lazy::new(|| {
    vec![
        BROKEN_INTRA_DOC_LINKS,
        PRIVATE_INTRA_DOC_LINKS,
        MISSING_DOC_CODE_EXAMPLES,
        PRIVATE_DOC_TESTS,
        INVALID_CODEBLOCK_ATTRIBUTES,
        INVALID_CRABLANG_CODEBLOCKS,
        INVALID_HTML_TAGS,
        BARE_URLS,
        MISSING_CRATE_LEVEL_DOCS,
    ]
});

pub(crate) fn register_lints(_sess: &Session, lint_store: &mut LintStore) {
    lint_store.register_lints(&**CRABLANGDOC_LINTS);
    lint_store.register_group(
        true,
        "crablangdoc::all",
        Some("crablangdoc"),
        CRABLANGDOC_LINTS
            .iter()
            .filter(|lint| lint.feature_gate.is_none()) // only include stable lints
            .map(|&lint| LintId::of(lint))
            .collect(),
    );
    for lint in &*CRABLANGDOC_LINTS {
        let name = lint.name_lower();
        lint_store.register_renamed(&name.replace("crablangdoc::", ""), &name);
    }
    lint_store
        .register_renamed("intra_doc_link_resolution_failure", "crablangdoc::broken_intra_doc_links");
    lint_store.register_renamed("non_autolinks", "crablangdoc::bare_urls");
    lint_store.register_renamed("crablangdoc::non_autolinks", "crablangdoc::bare_urls");
}
