//! Interacts with the registry [search API][1].
//!
//! [1]: https://doc.rust-lang.org/nightly/cargo/reference/registry-web-api.html#search

use std::cmp;
use std::iter::repeat;

use anyhow::Context as _;
use termcolor::Color;
use termcolor::ColorSpec;
use url::Url;

use crate::util::truncate_with_ellipsis;
use crate::CargoResult;
use crate::Config;

pub fn search(
    query: &str,
    config: &Config,
    index: Option<String>,
    limit: u32,
    reg: Option<String>,
) -> CargoResult<()> {
    let (mut registry, source_ids) =
        super::registry(config, None, index.as_deref(), reg.as_deref(), false, None)?;
    let (crates, total_crates) = registry.search(query, limit).with_context(|| {
        format!(
            "failed to retrieve search results from the registry at {}",
            registry.host()
        )
    })?;

    let names = crates
        .iter()
        .map(|krate| format!("{} = \"{}\"", krate.name, krate.max_version))
        .collect::<Vec<String>>();

    let description_margin = names.iter().map(|s| s.len() + 4).max().unwrap_or_default();

    let description_length = cmp::max(80, 128 - description_margin);

    let descriptions = crates.iter().map(|krate| {
        krate
            .description
            .as_ref()
            .map(|desc| truncate_with_ellipsis(&desc.replace("\n", " "), description_length))
    });

    for (name, description) in names.into_iter().zip(descriptions) {
        let line = match description {
            Some(desc) => {
                let space = repeat(' ')
                    .take(description_margin - name.len())
                    .collect::<String>();
                name + &space + "# " + &desc
            }
            None => name,
        };
        let mut fragments = line.split(query).peekable();
        while let Some(fragment) = fragments.next() {
            let _ = config.shell().write_stdout(fragment, &ColorSpec::new());
            if fragments.peek().is_some() {
                let _ = config.shell().write_stdout(
                    query,
                    &ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)),
                );
            }
        }
        let _ = config.shell().write_stdout("\n", &ColorSpec::new());
    }

    let search_max_limit = 100;
    if total_crates > limit && limit < search_max_limit {
        let _ = config.shell().write_stdout(
            format_args!(
                "... and {} crates more (use --limit N to see more)\n",
                total_crates - limit
            ),
            &ColorSpec::new(),
        );
    } else if total_crates > limit && limit >= search_max_limit {
        let extra = if source_ids.original.is_crates_io() {
            let url = Url::parse_with_params("https://crates.io/search", &[("q", query)])?;
            format!(" (go to {url} to see more)")
        } else {
            String::new()
        };
        let _ = config.shell().write_stdout(
            format_args!("... and {} crates more{}\n", total_crates - limit, extra),
            &ColorSpec::new(),
        );
    }

    Ok(())
}
