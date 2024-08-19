// Based on from https://github.com/rust-lang/rust/blob/2cb0b8582ebbf9784db9cec06fff517badbf4553/src/test/ui/issues/issue-45731.rs
// This needs to go in a crate by itself, since it modifies the dSYM for the entire test
// output directory.
//
// Note that this crate is *not* part of the overall `backtrace-rs` workspace,
// so that it gets its own 'target' directory. We manually invoke this test
// in .github/workflows/main.yml by passing `--manifest-path` to Cargo
#[test]
#[cfg(target_vendor = "apple")]
fn backtrace_no_dsym() {
    use std::{env, fs};

    // Find our dSYM and replace the DWARF binary with an empty file
    let mut dsym_path = env::current_exe().unwrap();
    let executable_name = dsym_path.file_name().unwrap().to_str().unwrap().to_string();
    assert!(dsym_path.pop()); // Pop executable
    dsym_path.push(format!(
        "{executable_name}.dSYM/Contents/Resources/DWARF/{executable_name}"
    ));
    let _ = fs::OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(false)
        .open(&dsym_path)
        .unwrap();

    backtrace::Backtrace::new();
}
