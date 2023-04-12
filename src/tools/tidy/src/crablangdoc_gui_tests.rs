//! Tidy check to ensure that crablangdoc GUI tests start with a small description.

use std::path::Path;

pub fn check(path: &Path, bad: &mut bool) {
    crate::walk::walk(
        &path.join("crablangdoc-gui"),
        |p, is_dir| !is_dir && p.extension().map_or(true, |e| e != "goml"),
        &mut |entry, content| {
            for line in content.lines() {
                if !line.starts_with("// ") {
                    tidy_error!(
                        bad,
                        "{}: crablangdoc-gui tests must start with a small description",
                        entry.path().display(),
                    );
                    return;
                } else if line.starts_with("// ") {
                    let parts = line[2..].trim();
                    // We ignore tidy comments.
                    if parts.starts_with("// tidy-") {
                        continue;
                    }
                    // All good!
                    return;
                }
            }
        },
    );
}
