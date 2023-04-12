//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary defines various auxiliary build commands, which are not
//! expressible with just `cargo`. Notably, it provides tests via `cargo test -p xtask`
//! for code generation and `cargo xtask install` for installation of
//! crablang-analyzer server and client.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`.

#![warn(crablang_2018_idioms, unused_lifetimes, semicolon_in_expressions_from_macros)]

mod flags;

mod install;
mod release;
mod dist;
mod publish;
mod metrics;

use anyhow::bail;
use std::{
    env,
    path::{Path, PathBuf},
};
use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let flags = flags::Xtask::from_env_or_exit();

    let sh = &Shell::new()?;
    sh.change_dir(project_root());

    match flags.subcommand {
        flags::XtaskCmd::Install(cmd) => cmd.run(sh),
        flags::XtaskCmd::FuzzTests(_) => run_fuzzer(sh),
        flags::XtaskCmd::Release(cmd) => cmd.run(sh),
        flags::XtaskCmd::Promote(cmd) => cmd.run(sh),
        flags::XtaskCmd::Dist(cmd) => cmd.run(sh),
        flags::XtaskCmd::PublishReleaseNotes(cmd) => cmd.run(sh),
        flags::XtaskCmd::Metrics(cmd) => cmd.run(sh),
        flags::XtaskCmd::Bb(cmd) => {
            {
                let _d = sh.push_dir("./crates/crablang-analyzer");
                cmd!(sh, "cargo build --release --features jemalloc").run()?;
            }
            sh.copy_file(
                "./target/release/crablang-analyzer",
                format!("./target/crablang-analyzer-{}", cmd.suffix),
            )?;
            Ok(())
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

fn run_fuzzer(sh: &Shell) -> anyhow::Result<()> {
    let _d = sh.push_dir("./crates/syntax");
    let _e = sh.push_env("CRABLANGUP_TOOLCHAIN", "nightly");
    if cmd!(sh, "cargo fuzz --help").read().is_err() {
        cmd!(sh, "cargo install cargo-fuzz").run()?;
    };

    // Expecting nightly crablangc
    let out = cmd!(sh, "crablangc --version").read()?;
    if !out.contains("nightly") {
        bail!("fuzz tests require nightly crablangc")
    }

    cmd!(sh, "cargo fuzz run parser").run()?;
    Ok(())
}

fn date_iso(sh: &Shell) -> anyhow::Result<String> {
    let res = cmd!(sh, "date -u +%Y-%m-%d").read()?;
    Ok(res)
}

fn is_release_tag(tag: &str) -> bool {
    tag.len() == "2020-02-24".len() && tag.starts_with(|c: char| c.is_ascii_digit())
}
