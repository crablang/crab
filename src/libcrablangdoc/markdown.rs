use std::fmt::Write as _;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::prelude::*;
use std::path::Path;

use crablangc_span::edition::Edition;
use crablangc_span::source_map::DUMMY_SP;

use crate::config::{Options, RenderOptions};
use crate::doctest::{Collector, GlobalTestOptions};
use crate::html::escape::Escape;
use crate::html::markdown;
use crate::html::markdown::{
    find_testable_code, ErrorCodes, HeadingOffset, IdMap, Markdown, MarkdownWithToc,
};

/// Separate any lines at the start of the file that begin with `# ` or `%`.
fn extract_leading_metadata(s: &str) -> (Vec<&str>, &str) {
    let mut metadata = Vec::new();
    let mut count = 0;

    for line in s.lines() {
        if line.starts_with("# ") || line.starts_with('%') {
            // trim the whitespace after the symbol
            metadata.push(line[1..].trim_start());
            count += line.len() + 1;
        } else {
            return (metadata, &s[count..]);
        }
    }

    // if we're here, then all lines were metadata `# ` or `%` lines.
    (metadata, "")
}

/// Render `input` (e.g., "foo.md") into an HTML file in `output`
/// (e.g., output = "bar" => "bar/foo.html").
///
/// Requires session globals to be available, for symbol interning.
pub(crate) fn render<P: AsRef<Path>>(
    input: P,
    options: RenderOptions,
    edition: Edition,
) -> Result<(), String> {
    if let Err(e) = create_dir_all(&options.output) {
        return Err(format!("{}: {}", options.output.display(), e));
    }

    let input = input.as_ref();
    let mut output = options.output;
    output.push(input.file_name().unwrap());
    output.set_extension("html");

    let mut css = String::new();
    for name in &options.markdown_css {
        write!(css, r#"<link rel="stylesheet" href="{name}">"#)
            .expect("Writing to a String can't fail");
    }

    let input_str = read_to_string(input).map_err(|err| format!("{}: {}", input.display(), err))?;
    let playground_url = options.markdown_playground_url.or(options.playground_url);
    let playground = playground_url.map(|url| markdown::Playground { crate_name: None, url });

    let mut out = File::create(&output).map_err(|e| format!("{}: {}", output.display(), e))?;

    let (metadata, text) = extract_leading_metadata(&input_str);
    if metadata.is_empty() {
        return Err("invalid markdown file: no initial lines starting with `# ` or `%`".to_owned());
    }
    let title = metadata[0];

    let mut ids = IdMap::new();
    let error_codes = ErrorCodes::from(options.unstable_features.is_nightly_build());
    let text = if !options.markdown_no_toc {
        MarkdownWithToc {
            content: text,
            ids: &mut ids,
            error_codes,
            edition,
            playground: &playground,
        }
        .into_string()
    } else {
        Markdown {
            content: text,
            links: &[],
            ids: &mut ids,
            error_codes,
            edition,
            playground: &playground,
            heading_offset: HeadingOffset::H1,
        }
        .into_string()
    };

    let err = write!(
        &mut out,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="crablangdoc">
    <title>{title}</title>

    {css}
    {in_header}
</head>
<body class="crablangdoc">
    <!--[if lte IE 8]>
    <div class="warning">
        This old browser is unsupported and will most likely display funky
        things.
    </div>
    <![endif]-->

    {before_content}
    <h1 class="title">{title}</h1>
    {text}
    {after_content}
</body>
</html>"#,
        title = Escape(title),
        css = css,
        in_header = options.external_html.in_header,
        before_content = options.external_html.before_content,
        text = text,
        after_content = options.external_html.after_content,
    );

    match err {
        Err(e) => Err(format!("cannot write to `{}`: {}", output.display(), e)),
        Ok(_) => Ok(()),
    }
}

/// Runs any tests/code examples in the markdown file `input`.
pub(crate) fn test(options: Options) -> Result<(), String> {
    let input_str = read_to_string(&options.input)
        .map_err(|err| format!("{}: {}", options.input.display(), err))?;
    let mut opts = GlobalTestOptions::default();
    opts.no_crate_inject = true;
    let mut collector = Collector::new(
        options.input.display().to_string(),
        options.clone(),
        true,
        opts,
        None,
        Some(options.input),
        options.enable_per_target_ignores,
    );
    collector.set_position(DUMMY_SP);
    let codes = ErrorCodes::from(options.unstable_features.is_nightly_build());

    find_testable_code(&input_str, &mut collector, codes, options.enable_per_target_ignores, None);

    crate::doctest::run_tests(options.test_args, options.nocapture, collector.tests);
    Ok(())
}
