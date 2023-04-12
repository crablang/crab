//! Transforms markdown
use ide_db::crablang_doc::is_crablang_fence;

const CRABLANGDOC_FENCES: [&str; 2] = ["```", "~~~"];

pub(crate) fn format_docs(src: &str) -> String {
    let mut processed_lines = Vec::new();
    let mut in_code_block = false;
    let mut is_crablang = false;

    for mut line in src.lines() {
        if in_code_block && is_crablang && code_line_ignored_by_crablangdoc(line) {
            continue;
        }

        if let Some(header) = CRABLANGDOC_FENCES.into_iter().find_map(|fence| line.strip_prefix(fence))
        {
            in_code_block ^= true;

            if in_code_block {
                is_crablang = is_crablang_fence(header);

                if is_crablang {
                    line = "```crablang";
                }
            }
        }

        if in_code_block {
            let trimmed = line.trim_start();
            if trimmed.starts_with("##") {
                line = &trimmed[1..];
            }
        }

        processed_lines.push(line);
    }
    processed_lines.join("\n")
}

fn code_line_ignored_by_crablangdoc(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "#" || trimmed.starts_with("# ") || trimmed.starts_with("#\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_docs_adds_crablang() {
        let comment = "```\nfn some_crablang() {}\n```";
        assert_eq!(format_docs(comment), "```crablang\nfn some_crablang() {}\n```");
    }

    #[test]
    fn test_format_docs_handles_plain_text() {
        let comment = "```text\nthis is plain text\n```";
        assert_eq!(format_docs(comment), "```text\nthis is plain text\n```");
    }

    #[test]
    fn test_format_docs_handles_non_crablang() {
        let comment = "```sh\nsupposedly shell code\n```";
        assert_eq!(format_docs(comment), "```sh\nsupposedly shell code\n```");
    }

    #[test]
    fn test_format_docs_handles_crablang_alias() {
        let comment = "```ignore\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```crablang\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_handles_complex_code_block_attrs() {
        let comment = "```crablang,no_run\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```crablang\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_handles_error_codes() {
        let comment = "```compile_fail,E0641\nlet b = 0 as *const _;\n```";
        assert_eq!(format_docs(comment), "```crablang\nlet b = 0 as *const _;\n```");
    }

    #[test]
    fn test_format_docs_skips_comments_in_crablang_block() {
        let comment =
            "```crablang\n # skip1\n# skip2\n#stay1\nstay2\n#\n #\n   #    \n #\tskip3\n\t#\t\n```";
        assert_eq!(format_docs(comment), "```crablang\n#stay1\nstay2\n```");
    }

    #[test]
    fn test_format_docs_does_not_skip_lines_if_plain_text() {
        let comment =
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```";
        assert_eq!(
            format_docs(comment),
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```",
        );
    }

    #[test]
    fn test_format_docs_keeps_comments_outside_of_crablang_block() {
        let comment = " # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_format_docs_preserves_newlines() {
        let comment = "this\nis\nmultiline";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_crablang() {
        let comment = r#"```crablang
fn main(){}
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```crablang\nfn main(){}\n```\nSome comment.\n```crablang\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_text() {
        let comment = r#"```text
filler
text
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```text\nfiller\ntext\n```\nSome comment.\n```crablang\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_format_docs_handles_escape_double_hashes() {
        let comment = r#"```crablang
let s = "foo
## bar # baz";
```"#;

        assert_eq!(format_docs(comment), "```crablang\nlet s = \"foo\n# bar # baz\";\n```");
    }
}
