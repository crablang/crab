//! CrabLangdoc specific doc comment handling

// stripped down version of https://github.com/crablang/crablang/blob/392ba2ba1a7d6c542d2459fb8133bebf62a4a423/src/libcrablangdoc/html/markdown.rs#L810-L933
pub fn is_crablang_fence(s: &str) -> bool {
    let mut seen_crablang_tags = false;
    let mut seen_other_tags = false;

    let tokens = s
        .trim()
        .split(|c| c == ',' || c == ' ' || c == '\t')
        .map(str::trim)
        .filter(|t| !t.is_empty());

    for token in tokens {
        match token {
            "should_panic" | "no_run" | "ignore" | "allow_fail" => {
                seen_crablang_tags = !seen_other_tags
            }
            "crablang" => seen_crablang_tags = true,
            "test_harness" | "compile_fail" => seen_crablang_tags = !seen_other_tags || seen_crablang_tags,
            x if x.starts_with("edition") => {}
            x if x.starts_with('E') && x.len() == 5 => {
                if x[1..].parse::<u32>().is_ok() {
                    seen_crablang_tags = !seen_other_tags || seen_crablang_tags;
                } else {
                    seen_other_tags = true;
                }
            }
            _ => seen_other_tags = true,
        }
    }

    !seen_other_tags || seen_crablang_tags
}
