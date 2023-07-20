use anyhow::Context as _;

use crate::util::restricted_names;
use crate::CargoResult;
use crate::Config;

const DEFAULT_EDITION: crate::core::features::Edition =
    crate::core::features::Edition::LATEST_STABLE;
const DEFAULT_VERSION: &str = "0.0.0";
const DEFAULT_PUBLISH: bool = false;
const AUTO_FIELDS: &[&str] = &["autobins", "autoexamples", "autotests", "autobenches"];

pub fn expand_manifest(
    content: &str,
    path: &std::path::Path,
    config: &Config,
) -> CargoResult<String> {
    let comment = match extract_comment(content) {
        Ok(comment) => Some(comment),
        Err(err) => {
            log::trace!("failed to extract doc comment: {err}");
            None
        }
    }
    .unwrap_or_default();
    let manifest = match extract_manifest(&comment)? {
        Some(manifest) => Some(manifest),
        None => {
            log::trace!("failed to extract manifest");
            None
        }
    }
    .unwrap_or_default();
    let manifest = expand_manifest_(&manifest, path, config)
        .with_context(|| format!("failed to parse manifest at {}", path.display()))?;
    let manifest = toml::to_string_pretty(&manifest)?;
    Ok(manifest)
}

fn expand_manifest_(
    manifest: &str,
    path: &std::path::Path,
    config: &Config,
) -> CargoResult<toml::Table> {
    let mut manifest: toml::Table = toml::from_str(&manifest)?;

    for key in ["workspace", "lib", "bin", "example", "test", "bench"] {
        if manifest.contains_key(key) {
            anyhow::bail!("`{key}` is not allowed in embedded manifests")
        }
    }

    // Prevent looking for a workspace by `read_manifest_from_str`
    manifest.insert("workspace".to_owned(), toml::Table::new().into());

    let package = manifest
        .entry("package".to_owned())
        .or_insert_with(|| toml::Table::new().into())
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`package` must be a table"))?;
    for key in ["workspace", "build", "links"]
        .iter()
        .chain(AUTO_FIELDS.iter())
    {
        if package.contains_key(*key) {
            anyhow::bail!("`package.{key}` is not allowed in embedded manifests")
        }
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::format_err!("no file name"))?
        .to_string_lossy();
    let file_stem = path
        .file_stem()
        .ok_or_else(|| anyhow::format_err!("no file name"))?
        .to_string_lossy();
    let name = sanitize_name(file_stem.as_ref());
    let bin_name = name.clone();
    package
        .entry("name".to_owned())
        .or_insert(toml::Value::String(name));
    package
        .entry("version".to_owned())
        .or_insert_with(|| toml::Value::String(DEFAULT_VERSION.to_owned()));
    package.entry("edition".to_owned()).or_insert_with(|| {
        let _ = config.shell().warn(format_args!(
            "`package.edition` is unspecified, defaulting to `{}`",
            DEFAULT_EDITION
        ));
        toml::Value::String(DEFAULT_EDITION.to_string())
    });
    package
        .entry("build".to_owned())
        .or_insert_with(|| toml::Value::Boolean(false));
    package
        .entry("publish".to_owned())
        .or_insert_with(|| toml::Value::Boolean(DEFAULT_PUBLISH));
    for field in AUTO_FIELDS {
        package
            .entry(field.to_owned())
            .or_insert_with(|| toml::Value::Boolean(false));
    }

    let mut bin = toml::Table::new();
    bin.insert("name".to_owned(), toml::Value::String(bin_name));
    bin.insert(
        "path".to_owned(),
        toml::Value::String(file_name.into_owned()),
    );
    manifest.insert(
        "bin".to_owned(),
        toml::Value::Array(vec![toml::Value::Table(bin)]),
    );

    let release = manifest
        .entry("profile".to_owned())
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`profile` must be a table"))?
        .entry("release".to_owned())
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`profile.release` must be a table"))?;
    release
        .entry("strip".to_owned())
        .or_insert_with(|| toml::Value::Boolean(true));

    Ok(manifest)
}

/// Ensure the package name matches the validation from `ops::cargo_new::check_name`
fn sanitize_name(name: &str) -> String {
    let placeholder = if name.contains('_') {
        '_'
    } else {
        // Since embedded manifests only support `[[bin]]`s, prefer arrow-case as that is the
        // more common convention for CLIs
        '-'
    };

    let mut name = restricted_names::sanitize_package_name(name, placeholder);

    loop {
        if restricted_names::is_keyword(&name) {
            name.push(placeholder);
        } else if restricted_names::is_conflicting_artifact_name(&name) {
            // Being an embedded manifest, we always assume it is a `[[bin]]`
            name.push(placeholder);
        } else if name == "test" {
            name.push(placeholder);
        } else if restricted_names::is_windows_reserved(&name) {
            // Go ahead and be consistent across platforms
            name.push(placeholder);
        } else {
            break;
        }
    }

    name
}

/// Locates a "code block manifest" in Rust source.
fn extract_comment(input: &str) -> CargoResult<String> {
    let mut doc_fragments = Vec::new();
    let file = syn::parse_file(input)?;
    // HACK: `syn` doesn't tell us what kind of comment was used, so infer it from how many
    // attributes were used
    let kind = if 1 < file
        .attrs
        .iter()
        .filter(|attr| attr.meta.path().is_ident("doc"))
        .count()
    {
        CommentKind::Line
    } else {
        CommentKind::Block
    };
    for attr in &file.attrs {
        if attr.meta.path().is_ident("doc") {
            doc_fragments.push(DocFragment::new(attr, kind)?);
        }
    }
    if doc_fragments.is_empty() {
        anyhow::bail!("no doc-comment found");
    }
    unindent_doc_fragments(&mut doc_fragments);

    let mut doc_comment = String::new();
    for frag in &doc_fragments {
        add_doc_fragment(&mut doc_comment, frag);
    }

    Ok(doc_comment)
}

/// A `#[doc]`
#[derive(Clone, Debug)]
struct DocFragment {
    /// The attribute value
    doc: String,
    /// Indentation used within `doc
    indent: usize,
}

impl DocFragment {
    fn new(attr: &syn::Attribute, kind: CommentKind) -> CargoResult<Self> {
        let syn::Meta::NameValue(nv) = &attr.meta else {
            anyhow::bail!("unsupported attr meta for {:?}", attr.meta.path())
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = &nv.value
        else {
            anyhow::bail!("only string literals are supported")
        };
        Ok(Self {
            doc: beautify_doc_string(lit.value(), kind),
            indent: 0,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommentKind {
    Line,
    Block,
}

/// Makes a doc string more presentable to users.
/// Used by rustdoc and perhaps other tools, but not by rustc.
///
/// See `rustc_ast/util/comments.rs`
fn beautify_doc_string(data: String, kind: CommentKind) -> String {
    fn get_vertical_trim(lines: &[&str]) -> Option<(usize, usize)> {
        let mut i = 0;
        let mut j = lines.len();
        // first line of all-stars should be omitted
        if !lines.is_empty() && lines[0].chars().all(|c| c == '*') {
            i += 1;
        }

        // like the first, a last line of all stars should be omitted
        if j > i && !lines[j - 1].is_empty() && lines[j - 1].chars().all(|c| c == '*') {
            j -= 1;
        }

        if i != 0 || j != lines.len() {
            Some((i, j))
        } else {
            None
        }
    }

    fn get_horizontal_trim(lines: &[&str], kind: CommentKind) -> Option<String> {
        let mut i = usize::MAX;
        let mut first = true;

        // In case we have doc comments like `/**` or `/*!`, we want to remove stars if they are
        // present. However, we first need to strip the empty lines so they don't get in the middle
        // when we try to compute the "horizontal trim".
        let lines = match kind {
            CommentKind::Block => {
                // Whatever happens, we skip the first line.
                let mut i = lines
                    .get(0)
                    .map(|l| {
                        if l.trim_start().starts_with('*') {
                            0
                        } else {
                            1
                        }
                    })
                    .unwrap_or(0);
                let mut j = lines.len();

                while i < j && lines[i].trim().is_empty() {
                    i += 1;
                }
                while j > i && lines[j - 1].trim().is_empty() {
                    j -= 1;
                }
                &lines[i..j]
            }
            CommentKind::Line => lines,
        };

        for line in lines {
            for (j, c) in line.chars().enumerate() {
                if j > i || !"* \t".contains(c) {
                    return None;
                }
                if c == '*' {
                    if first {
                        i = j;
                        first = false;
                    } else if i != j {
                        return None;
                    }
                    break;
                }
            }
            if i >= line.len() {
                return None;
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines[0][..i].into())
        }
    }

    let data_s = data.as_str();
    if data_s.contains('\n') {
        let mut lines = data_s.lines().collect::<Vec<&str>>();
        let mut changes = false;
        let lines = if let Some((i, j)) = get_vertical_trim(&lines) {
            changes = true;
            // remove whitespace-only lines from the start/end of lines
            &mut lines[i..j]
        } else {
            &mut lines
        };
        if let Some(horizontal) = get_horizontal_trim(lines, kind) {
            changes = true;
            // remove a "[ \t]*\*" block from each line, if possible
            for line in lines.iter_mut() {
                if let Some(tmp) = line.strip_prefix(&horizontal) {
                    *line = tmp;
                    if kind == CommentKind::Block
                        && (*line == "*" || line.starts_with("* ") || line.starts_with("**"))
                    {
                        *line = &line[1..];
                    }
                }
            }
        }
        if changes {
            return lines.join("\n");
        }
    }
    data
}

/// Removes excess indentation on comments in order for the Markdown
/// to be parsed correctly. This is necessary because the convention for
/// writing documentation is to provide a space between the /// or //! marker
/// and the doc text, but Markdown is whitespace-sensitive. For example,
/// a block of text with four-space indentation is parsed as a code block,
/// so if we didn't unindent comments, these list items
///
/// /// A list:
/// ///
/// ///    - Foo
/// ///    - Bar
///
/// would be parsed as if they were in a code block, which is likely not what the user intended.
///
/// See also `rustc_resolve/rustdoc.rs`
fn unindent_doc_fragments(docs: &mut [DocFragment]) {
    // HACK: We can't tell the difference between `#[doc]` and doc-comments, so we can't specialize
    // the indentation like rustodc does
    let add = 0;

    // `min_indent` is used to know how much whitespaces from the start of each lines must be
    // removed. Example:
    //
    // ```
    // ///     hello!
    // #[doc = "another"]
    // ```
    //
    // In here, the `min_indent` is 1 (because non-sugared fragment are always counted with minimum
    // 1 whitespace), meaning that "hello!" will be considered a codeblock because it starts with 4
    // (5 - 1) whitespaces.
    let Some(min_indent) = docs
        .iter()
        .map(|fragment| {
            fragment
                .doc
                .as_str()
                .lines()
                .fold(usize::MAX, |min_indent, line| {
                    if line.chars().all(|c| c.is_whitespace()) {
                        min_indent
                    } else {
                        // Compare against either space or tab, ignoring whether they are
                        // mixed or not.
                        let whitespace =
                            line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                        min_indent.min(whitespace)
                    }
                })
        })
        .min()
    else {
        return;
    };

    for fragment in docs {
        if fragment.doc.is_empty() {
            continue;
        }

        let min_indent = if min_indent > 0 {
            min_indent - add
        } else {
            min_indent
        };

        fragment.indent = min_indent;
    }
}

/// The goal of this function is to apply the `DocFragment` transformation that is required when
/// transforming into the final Markdown, which is applying the computed indent to each line in
/// each doc fragment (a `DocFragment` can contain multiple lines in case of `#[doc = ""]`).
///
/// Note: remove the trailing newline where appropriate
///
/// See also `rustc_resolve/rustdoc.rs`
fn add_doc_fragment(out: &mut String, frag: &DocFragment) {
    let s = frag.doc.as_str();
    let mut iter = s.lines();
    if s.is_empty() {
        out.push('\n');
        return;
    }
    while let Some(line) = iter.next() {
        if line.chars().any(|c| !c.is_whitespace()) {
            assert!(line.len() >= frag.indent);
            out.push_str(&line[frag.indent..]);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
}

/// Extracts the first `Cargo` fenced code block from a chunk of Markdown.
fn extract_manifest(comment: &str) -> CargoResult<Option<String>> {
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

    // To match librustdoc/html/markdown.rs, opts.
    let exts = Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES;

    let md = Parser::new_ext(comment, exts);

    let mut inside = false;
    let mut output = None;

    for item in md {
        match item {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info)))
                if info.to_lowercase() == "cargo" =>
            {
                if output.is_some() {
                    anyhow::bail!("multiple `cargo` manifests present")
                } else {
                    output = Some(String::new());
                }
                inside = true;
            }
            Event::Text(ref text) if inside => {
                let s = output.get_or_insert(String::new());
                s.push_str(text);
            }
            Event::End(Tag::CodeBlock(_)) if inside => {
                inside = false;
            }
            _ => (),
        }
    }

    Ok(output)
}

#[cfg(test)]
mod test_expand {
    use super::*;

    macro_rules! si {
        ($i:expr) => {{
            expand_manifest(
                $i,
                std::path::Path::new("/home/me/test.rs"),
                &Config::default().unwrap(),
            )
            .unwrap_or_else(|err| panic!("{}", err))
        }};
    }

    #[test]
    fn test_default() {
        snapbox::assert_eq(
            r#"[[bin]]
name = "test-"
path = "test.rs"

[package]
autobenches = false
autobins = false
autoexamples = false
autotests = false
build = false
edition = "2021"
name = "test-"
publish = false
version = "0.0.0"

[profile.release]
strip = true

[workspace]
"#,
            si!(r#"fn main() {}"#),
        );
    }

    #[test]
    fn test_dependencies() {
        snapbox::assert_eq(
            r#"[[bin]]
name = "test-"
path = "test.rs"

[dependencies]
time = "0.1.25"

[package]
autobenches = false
autobins = false
autoexamples = false
autotests = false
build = false
edition = "2021"
name = "test-"
publish = false
version = "0.0.0"

[profile.release]
strip = true

[workspace]
"#,
            si!(r#"
//! ```cargo
//! [dependencies]
//! time="0.1.25"
//! ```
fn main() {}
"#),
        );
    }
}

#[cfg(test)]
mod test_comment {
    use super::*;

    macro_rules! ec {
        ($s:expr) => {
            extract_comment($s).unwrap_or_else(|err| panic!("{}", err))
        };
    }

    #[test]
    fn test_no_comment() {
        snapbox::assert_eq(
            "no doc-comment found",
            extract_comment(
                r#"
fn main () {
}
"#,
            )
            .unwrap_err()
            .to_string(),
        );
    }

    #[test]
    fn test_no_comment_she_bang() {
        snapbox::assert_eq(
            "no doc-comment found",
            extract_comment(
                r#"#!/usr/bin/env cargo-eval

fn main () {
}
"#,
            )
            .unwrap_err()
            .to_string(),
        );
    }

    #[test]
    fn test_comment() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"//! Here is a manifest:
//!
//! ```cargo
//! [dependencies]
//! time = "*"
//! ```
fn main() {}
"#),
        );
    }

    #[test]
    fn test_comment_shebang() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"#!/usr/bin/env cargo-eval

//! Here is a manifest:
//!
//! ```cargo
//! [dependencies]
//! time = "*"
//! ```
fn main() {}
"#),
        );
    }

    #[test]
    fn test_multiline_comment() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"/*!
Here is a manifest:

```cargo
[dependencies]
time = "*"
```
*/

fn main() {
}
"#),
        );
    }

    #[test]
    fn test_multiline_comment_shebang() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"#!/usr/bin/env cargo-eval

/*!
Here is a manifest:

```cargo
[dependencies]
time = "*"
```
*/

fn main() {
}
"#),
        );
    }

    #[test]
    fn test_multiline_block_comment() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"/*!
 * Here is a manifest:
 *
 * ```cargo
 * [dependencies]
 * time = "*"
 * ```
 */
fn main() {}
"#),
        );
    }

    #[test]
    fn test_multiline_block_comment_shebang() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"#!/usr/bin/env cargo-eval

/*!
 * Here is a manifest:
 *
 * ```cargo
 * [dependencies]
 * time = "*"
 * ```
 */
fn main() {}
"#),
        );
    }

    #[test]
    fn test_adjacent_comments() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r#"#!/usr/bin/env cargo-eval

// I am a normal comment
//! Here is a manifest:
//!
//! ```cargo
//! [dependencies]
//! time = "*"
//! ```

fn main () {
}
"#),
        );
    }

    #[test]
    fn test_doc_attrib() {
        snapbox::assert_eq(
            r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#,
            ec!(r###"#!/usr/bin/env cargo-eval

#![doc = r#"Here is a manifest:

```cargo
[dependencies]
time = "*"
```
"#]

fn main () {
}
"###),
        );
    }
}

#[cfg(test)]
mod test_manifest {
    use super::*;

    macro_rules! smm {
        ($c:expr) => {
            extract_manifest($c)
        };
    }

    #[test]
    fn test_no_code_fence() {
        assert_eq!(
            smm!(
                r#"There is no manifest in this comment.
"#
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn test_no_cargo_code_fence() {
        assert_eq!(
            smm!(
                r#"There is no manifest in this comment.

```
This is not a manifest.
```

```rust
println!("Nor is this.");
```

    Or this.
"#
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn test_cargo_code_fence() {
        assert_eq!(
            smm!(
                r#"This is a manifest:

```cargo
dependencies = { time = "*" }
```
"#
            )
            .unwrap(),
            Some(
                r#"dependencies = { time = "*" }
"#
                .into()
            )
        );
    }

    #[test]
    fn test_mixed_code_fence() {
        assert_eq!(
            smm!(
                r#"This is *not* a manifest:

```
He's lying, I'm *totally* a manifest!
```

This *is*:

```cargo
dependencies = { time = "*" }
```
"#
            )
            .unwrap(),
            Some(
                r#"dependencies = { time = "*" }
"#
                .into()
            )
        );
    }

    #[test]
    fn test_two_cargo_code_fence() {
        assert!(smm!(
            r#"This is a manifest:

```cargo
dependencies = { time = "*" }
```

So is this, but it doesn't count:

```cargo
dependencies = { explode = true }
```
"#
        )
        .is_err());
    }
}
