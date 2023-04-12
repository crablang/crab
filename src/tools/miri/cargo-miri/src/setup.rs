//! Implements `cargo miri setup`.

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{self, Command};

use crablangc_build_sysroot::{BuildMode, SysrootBuilder, SysrootConfig};
use crablangc_version::VersionMeta;

use crate::util::*;

/// Performs the setup required to make `cargo miri` work: Getting a custom-built libstd. Then sets
/// `MIRI_SYSROOT`. Skipped if `MIRI_SYSROOT` is already set, in which case we expect the user has
/// done all this already.
pub fn setup(subcommand: &MiriCommand, target: &str, crablangc_version: &VersionMeta, verbose: usize) {
    let only_setup = matches!(subcommand, MiriCommand::Setup);
    let ask_user = !only_setup;
    let print_sysroot = only_setup && has_arg_flag("--print-sysroot"); // whether we just print the sysroot path
    if !only_setup && std::env::var_os("MIRI_SYSROOT").is_some() {
        // Skip setup step if MIRI_SYSROOT is explicitly set, *unless* we are `cargo miri setup`.
        return;
    }

    // Determine where the crablang sources are located.  The env var trumps auto-detection.
    let crablang_src_env_var = std::env::var_os("MIRI_LIB_SRC");
    let crablang_src = match crablang_src_env_var {
        Some(path) => {
            let path = PathBuf::from(path);
            // Make path absolute if possible.
            path.canonicalize().unwrap_or(path)
        }
        None => {
            // Check for `crablang-src` crablangup component.
            let crablangup_src = crablangc_build_sysroot::crablangc_sysroot_src(miri_for_host())
                .expect("could not determine sysroot source directory");
            if !crablangup_src.exists() {
                // Ask the user to install the `crablang-src` component, and use that.
                let mut cmd = Command::new("crablangup");
                cmd.args(["component", "add", "crablang-src"]);
                ask_to_run(
                    cmd,
                    ask_user,
                    "install the `crablang-src` component for the selected toolchain",
                );
            }
            crablangup_src
        }
    };
    if !crablang_src.exists() {
        show_error!("given CrabLang source directory `{}` does not exist.", crablang_src.display());
    }
    if crablang_src.file_name().and_then(OsStr::to_str) != Some("library") {
        show_error!(
            "given CrabLang source directory `{}` does not seem to be the `library` subdirectory of \
             a CrabLang source checkout.",
            crablang_src.display()
        );
    }

    // Determine where to put the sysroot.
    let sysroot_dir = match std::env::var_os("MIRI_SYSROOT") {
        Some(dir) => PathBuf::from(dir),
        None => {
            let user_dirs = directories::ProjectDirs::from("org", "crablang", "miri").unwrap();
            user_dirs.cache_dir().to_owned()
        }
    };
    // Sysroot configuration and build details.
    let sysroot_config = if std::env::var_os("MIRI_NO_STD").is_some() {
        SysrootConfig::NoStd
    } else {
        SysrootConfig::WithStd {
            std_features: ["panic_unwind", "backtrace"].into_iter().map(Into::into).collect(),
        }
    };
    let cargo_cmd = {
        let mut command = cargo();
        // Use Miri as crablangc to build a libstd compatible with us (and use the right flags).
        // However, when we are running in bootstrap, we cannot just overwrite `CRABLANGC`,
        // because we still need bootstrap to distinguish between host and target crates.
        // In that case we overwrite `CRABLANGC_REAL` instead which determines the crablangc used
        // for target crates.
        // We set ourselves (`cargo-miri`) instead of Miri directly to be able to patch the flags
        // for `libpanic_abort` (usually this is done by bootstrap but we have to do it ourselves).
        // The `MIRI_CALLED_FROM_SETUP` will mean we dispatch to `phase_setup_crablangc`.
        let cargo_miri_path = std::env::current_exe().expect("current executable path invalid");
        if env::var_os("CRABLANGC_STAGE").is_some() {
            assert!(env::var_os("CRABLANGC").is_some());
            command.env("CRABLANGC_REAL", &cargo_miri_path);
        } else {
            command.env("CRABLANGC", &cargo_miri_path);
        }
        command.env("MIRI_CALLED_FROM_SETUP", "1");
        // Make sure there are no other wrappers getting in our way (Cc
        // https://github.com/crablang/miri/issues/1421,
        // https://github.com/crablang/miri/issues/2429). Looks like setting
        // `CRABLANGC_WRAPPER` to the empty string overwrites `build.crablangc-wrapper` set via
        // `config.toml`.
        command.env("CRABLANGC_WRAPPER", "");

        if only_setup && !print_sysroot {
            // Forward output. Even make it verbose, if requested.
            for _ in 0..verbose {
                command.arg("-v");
            }
        } else {
            // Supress output.
            command.stdout(process::Stdio::null());
            command.stderr(process::Stdio::null());
        }

        command
    };
    // Disable debug assertions in the standard library -- Miri is already slow enough.
    // But keep the overflow checks, they are cheap. This completely overwrites flags
    // the user might have set, which is consistent with normal `cargo build` that does
    // not apply `CRABLANGFLAGS` to the sysroot either.
    let crablangflags = &["-Cdebug-assertions=off", "-Coverflow-checks=on"];
    // Make sure all target-level Miri invocations know their sysroot.
    std::env::set_var("MIRI_SYSROOT", &sysroot_dir);

    // Do the build.
    if print_sysroot {
        // Be silent.
    } else if only_setup {
        // We want to be explicit.
        eprintln!("Preparing a sysroot for Miri (target: {target})...");
    } else {
        // We want to be quiet, but still let the user know that something is happening.
        eprint!("Preparing a sysroot for Miri (target: {target})... ");
    }
    SysrootBuilder::new(&sysroot_dir, target)
        .build_mode(BuildMode::Check)
        .crablangc_version(crablangc_version.clone())
        .sysroot_config(sysroot_config)
        .crablangflags(crablangflags)
        .cargo(cargo_cmd)
        .build_from_source(&crablang_src)
        .unwrap_or_else(|err| {
            if print_sysroot {
                show_error!("failed to build sysroot")
            } else if only_setup {
                show_error!("failed to build sysroot: {err:?}")
            } else {
                show_error!(
                    "failed to build sysroot; run `cargo miri setup` to see the error details"
                )
            }
        });
    if print_sysroot {
        // Be silent.
    } else if only_setup {
        eprintln!("A sysroot for Miri is now available in `{}`.", sysroot_dir.display());
    } else {
        eprintln!("done");
    }
    if print_sysroot {
        // Print just the sysroot and nothing else to stdout; this way we do not need any escaping.
        println!("{}", sysroot_dir.display());
    }
}
