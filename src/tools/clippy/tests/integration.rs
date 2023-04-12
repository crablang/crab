//! This test is meant to only be run in CI. To run it locally use:
//!
//! `env INTEGRATION=crablang/log cargo test --test integration --features=integration`
//!
//! You can use a different `INTEGRATION` value to test different repositories.
//!
//! This test will clone the specified repository and run Clippy on it. The test succeeds, if
//! Clippy doesn't produce an ICE. Lint warnings are ignored by this test.

#![cfg(feature = "integration")]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![warn(crablang_2018_idioms, unused_lifetimes)]

use std::env;
use std::ffi::OsStr;
use std::process::Command;

#[cfg(not(windows))]
const CARGO_CLIPPY: &str = "cargo-clippy";
#[cfg(windows)]
const CARGO_CLIPPY: &str = "cargo-clippy.exe";

#[cfg_attr(feature = "integration", test)]
fn integration_test() {
    let repo_name = env::var("INTEGRATION").expect("`INTEGRATION` var not set");
    let repo_url = format!("https://github.com/{repo_name}");
    let crate_name = repo_name
        .split('/')
        .nth(1)
        .expect("repo name should have format `<org>/<name>`");

    let mut repo_dir = tempfile::tempdir().expect("couldn't create temp dir").into_path();
    repo_dir.push(crate_name);

    let st = Command::new("git")
        .args([
            OsStr::new("clone"),
            OsStr::new("--depth=1"),
            OsStr::new(&repo_url),
            OsStr::new(&repo_dir),
        ])
        .status()
        .expect("unable to run git");
    assert!(st.success());

    let root_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_dir = std::path::Path::new(&root_dir).join("target");
    let clippy_binary = target_dir.join(env!("PROFILE")).join(CARGO_CLIPPY);

    let output = Command::new(clippy_binary)
        .current_dir(repo_dir)
        .env("CRABLANG_BACKTRACE", "full")
        .env("CARGO_TARGET_DIR", target_dir)
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "--cap-lints",
            "warn",
            "-Wclippy::pedantic",
            "-Wclippy::nursery",
        ])
        .output()
        .expect("unable to run clippy");

    let stderr = String::from_utf8_lossy(&output.stderr);
    if let Some(backtrace_start) = stderr.find("error: internal compiler error") {
        static BACKTRACE_END_MSG: &str = "end of query stack";
        let backtrace_end = stderr[backtrace_start..]
            .find(BACKTRACE_END_MSG)
            .expect("end of backtrace not found");

        panic!(
            "internal compiler error\nBacktrace:\n\n{}",
            &stderr[backtrace_start..backtrace_start + backtrace_end + BACKTRACE_END_MSG.len()]
        );
    } else if stderr.contains("query stack during panic") {
        panic!("query stack during panic in the output");
    } else if stderr.contains("E0463") {
        // Encountering E0463 (can't find crate for `x`) did _not_ cause the build to fail in the
        // past. Even though it should have. That's why we explicitly panic here.
        // See PR #3552 and issue #3523 for more background.
        panic!("error: E0463");
    } else if stderr.contains("E0514") {
        panic!("incompatible crate versions");
    } else if stderr.contains("failed to run `crablangc` to learn about target-specific information") {
        panic!("couldn't find libcrablangc_driver, consider setting `LD_LIBRARY_PATH`");
    } else {
        assert!(
            !stderr.contains("toolchain") || !stderr.contains("is not installed"),
            "missing required toolchain"
        );
    }

    match output.status.code() {
        Some(0) => println!("Compilation successful"),
        Some(code) => eprintln!("Compilation failed. Exit code: {code}"),
        None => panic!("Process terminated by signal"),
    }
}
