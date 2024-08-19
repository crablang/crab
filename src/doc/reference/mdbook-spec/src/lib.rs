use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

mod std_links;

/// The Regex for rules like `r[foo]`.
static RULE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^r\[([^]]+)]$").unwrap());

/// The Regex for the syntax for blockquotes that have a specific CSS class,
/// like `> [!WARNING]`.
static ADMONITION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^ *> \[!(?<admon>[^]]+)\]\n(?<blockquote>(?: *> .*\n)+)").unwrap()
});

pub fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

pub struct Spec {
    /// Whether or not warnings should be errors (set by SPEC_DENY_WARNINGS
    /// environment variable).
    deny_warnings: bool,
}

impl Spec {
    pub fn new() -> Spec {
        Spec {
            deny_warnings: std::env::var("SPEC_DENY_WARNINGS").as_deref() == Ok("1"),
        }
    }

    /// Converts lines that start with `r[…]` into a "rule" which has special
    /// styling and can be linked to.
    fn rule_definitions(
        &self,
        chapter: &Chapter,
        found_rules: &mut BTreeMap<String, (PathBuf, PathBuf)>,
    ) -> String {
        let source_path = chapter.source_path.clone().unwrap_or_default();
        let path = chapter.path.clone().unwrap_or_default();
        RULE_RE
            .replace_all(&chapter.content, |caps: &Captures<'_>| {
                let rule_id = &caps[1];
                if let Some((old, _)) =
                    found_rules.insert(rule_id.to_string(), (source_path.clone(), path.clone()))
                {
                    let message = format!(
                        "rule `{rule_id}` defined multiple times\n\
                        First location: {old:?}\n\
                        Second location: {source_path:?}"
                    );
                    if self.deny_warnings {
                        panic!("error: {message}");
                    } else {
                        eprintln!("warning: {message}");
                    }
                }
                format!(
                    "<div class=\"rule\" id=\"{rule_id}\">\
                     <a class=\"rule-link\" href=\"#{rule_id}\">[{rule_id}]</a>\
                     </div>\n"
                )
            })
            .to_string()
    }

    /// Generates link references to all rules on all pages, so you can easily
    /// refer to rules anywhere in the book.
    fn auto_link_references(
        &self,
        chapter: &Chapter,
        found_rules: &BTreeMap<String, (PathBuf, PathBuf)>,
    ) -> String {
        let current_path = chapter.path.as_ref().unwrap().parent().unwrap();
        let definitions: String = found_rules
            .iter()
            .map(|(rule_id, (_, path))| {
                let relative = pathdiff::diff_paths(path, current_path).unwrap();
                format!("[{rule_id}]: {}#{rule_id}\n", relative.display())
            })
            .collect();
        format!(
            "{}\n\
            {definitions}",
            chapter.content
        )
    }

    /// Converts blockquotes with special headers into admonitions.
    ///
    /// The blockquote should look something like:
    ///
    /// ```markdown
    /// > [!WARNING]
    /// > ...
    /// ```
    ///
    /// This will add a `<div class="warning">` around the blockquote so that
    /// it can be styled differently. Any text between the brackets that can
    /// be a CSS class is valid. The actual styling needs to be added in a CSS
    /// file.
    fn admonitions(&self, chapter: &Chapter) -> String {
        ADMONITION_RE
            .replace_all(&chapter.content, |caps: &Captures<'_>| {
                let lower = caps["admon"].to_lowercase();
                format!(
                    "<div class=\"{lower}\">\n\n{}\n\n</div>\n",
                    &caps["blockquote"]
                )
            })
            .to_string()
    }
}

impl Preprocessor for Spec {
    fn name(&self) -> &str {
        "spec"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut found_rules = BTreeMap::new();
        book.for_each_mut(|item| {
            let BookItem::Chapter(ch) = item else {
                return;
            };
            if ch.is_draft_chapter() {
                return;
            }
            ch.content = self.rule_definitions(&ch, &mut found_rules);
            ch.content = self.admonitions(&ch);
            ch.content = std_links::std_links(&ch);
        });
        // This is a separate pass because it relies on the modifications of
        // the previous passes.
        book.for_each_mut(|item| {
            let BookItem::Chapter(ch) = item else {
                return;
            };
            if ch.is_draft_chapter() {
                return;
            }
            ch.content = self.auto_link_references(&ch, &found_rules);
        });
        Ok(book)
    }
}
