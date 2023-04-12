//! Implements the various phases of `cargo miri run/test`.

use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

use crablangc_version::VersionMeta;

use crate::{setup::*, util::*};

const CARGO_MIRI_HELP: &str = r#"Runs binary crates and tests in Miri

Usage:
    cargo miri [subcommand] [<cargo options>...] [--] [<program/test suite options>...]

Subcommands:
    run, r                   Run binaries
    test, t                  Run tests
    nextest                  Run tests with nextest (requires cargo-nextest installed)
    setup                    Only perform automatic setup, but without asking questions (for getting a proper libstd)

The cargo options are exactly the same as for `cargo run` and `cargo test`, respectively.

Examples:
    cargo miri run
    cargo miri test -- test-suite-filter

    cargo miri setup --print sysroot
        This will print the path to the generated sysroot (and nothing else) on stdout.
        stderr will still contain progress information about how the build is doing.

"#;

fn show_help() {
    println!("{CARGO_MIRI_HELP}");
}

fn show_version() {
    print!("miri {}", env!("CARGO_PKG_VERSION"));
    let version = format!("{} {}", env!("GIT_HASH"), env!("COMMIT_DATE"));
    if version.len() > 1 {
        // If there is actually something here, print it.
        print!(" ({version})");
    }
    println!();
}

fn forward_patched_extern_arg(args: &mut impl Iterator<Item = String>, cmd: &mut Command) {
    cmd.arg("--extern"); // always forward flag, but adjust filename:
    let path = args.next().expect("`--extern` should be followed by a filename");
    if let Some(lib) = path.strip_suffix(".rlib") {
        // If this is an rlib, make it an rmeta.
        cmd.arg(format!("{lib}.rmeta"));
    } else {
        // Some other extern file (e.g. a `.so`). Forward unchanged.
        cmd.arg(path);
    }
}

pub fn phase_cargo_miri(mut args: impl Iterator<Item = String>) {
    // Check for version and help flags even when invoked as `cargo-miri`.
    if has_arg_flag("--help") || has_arg_flag("-h") {
        show_help();
        return;
    }
    if has_arg_flag("--version") || has_arg_flag("-V") {
        show_version();
        return;
    }

    // Require a subcommand before any flags.
    // We cannot know which of those flags take arguments and which do not,
    // so we cannot detect subcommands later.
    let Some(subcommand) = args.next() else {
        show_error!("`cargo miri` needs to be called with a subcommand (`run`, `test`)");
    };
    let subcommand = match &*subcommand {
        "setup" => MiriCommand::Setup,
        "test" | "t" | "run" | "r" | "nextest" => MiriCommand::Forward(subcommand),
        _ =>
            show_error!(
                "`cargo miri` supports the following subcommands: `run`, `test`, `nextest`, and `setup`."
            ),
    };
    let verbose = num_arg_flag("-v");

    // Determine the involved architectures.
    let crablangc_version = VersionMeta::for_command(miri_for_host())
        .expect("failed to determine underlying crablangc version of Miri");
    let host = &crablangc_version.host;
    let target = get_arg_flag_value("--target");
    let target = target.as_ref().unwrap_or(host);

    // We always setup.
    setup(&subcommand, target, &crablangc_version, verbose);

    // Invoke actual cargo for the job, but with different flags.
    // We re-use `cargo test` and `cargo run`, which makes target and binary handling very easy but
    // requires some extra work to make the build check-only (see all the `--emit` hacks below).
    // <https://github.com/crablang/miri/pull/1540#issuecomment-693553191> describes an alternative
    // approach that uses `cargo check`, making that part easier but target and binary handling
    // harder.
    let cargo_miri_path = std::env::current_exe()
        .expect("current executable path invalid")
        .into_os_string()
        .into_string()
        .expect("current executable path is not valid UTF-8");
    let cargo_cmd = match subcommand {
        MiriCommand::Forward(s) => s,
        MiriCommand::Setup => return, // `cargo miri setup` stops here.
    };
    let metadata = get_cargo_metadata();
    let mut cmd = cargo();
    cmd.arg(cargo_cmd);

    // Forward all arguments before `--` other than `--target-dir` and its value to Cargo.
    // (We want to *change* the target-dir value, so we must not forward it.)
    let mut target_dir = None;
    for arg in ArgSplitFlagValue::from_string_iter(&mut args, "--target-dir") {
        match arg {
            Ok(value) => {
                if target_dir.is_some() {
                    show_error!("`--target-dir` is provided more than once");
                }
                target_dir = Some(value.into());
            }
            Err(arg) => {
                cmd.arg(arg);
            }
        }
    }
    // Detect the target directory if it's not specified via `--target-dir`.
    // (`cargo metadata` does not support `--target-dir`, that's why we have to handle this ourselves.)
    let target_dir = target_dir.get_or_insert_with(|| metadata.target_directory.clone());
    // Set `--target-dir` to `miri` inside the original target directory.
    target_dir.push("miri");
    cmd.arg("--target-dir").arg(target_dir);

    // Make sure the build target is explicitly set.
    // This is needed to make the `target.runner` settings do something,
    // and it later helps us detect which crates are proc-macro/build-script
    // (host crates) and which crates are needed for the program itself.
    if get_arg_flag_value("--target").is_none() {
        // No target given. Explicitly pick the host.
        cmd.arg("--target");
        cmd.arg(host);
    }

    // Set ourselves as runner for al binaries invoked by cargo.
    // We use `all()` since `true` is not a thing in cfg-lang, but the empty conjunction is. :)
    let cargo_miri_path_for_toml = escape_for_toml(&cargo_miri_path);
    cmd.arg("--config")
        .arg(format!("target.'cfg(all())'.runner=[{cargo_miri_path_for_toml}, 'runner']"));

    // Forward all further arguments after `--` to cargo.
    cmd.arg("--").args(args);

    // Set `CRABLANGC_WRAPPER` to ourselves.  Cargo will prepend that binary to its usual invocation,
    // i.e., the first argument is `crablangc` -- which is what we use in `main` to distinguish
    // the two codepaths. (That extra argument is why we prefer this over setting `CRABLANGC`.)
    if env::var_os("CRABLANGC_WRAPPER").is_some() {
        println!(
            "WARNING: Ignoring `CRABLANGC_WRAPPER` environment variable, Miri does not support wrapping."
        );
    }
    cmd.env("CRABLANGC_WRAPPER", &cargo_miri_path);
    // We are going to invoke `MIRI` for everything, not `CRABLANGC`.
    if env::var_os("CRABLANGC").is_some() && env::var_os("MIRI").is_none() {
        println!(
            "WARNING: Ignoring `CRABLANGC` environment variable; set `MIRI` if you want to control the binary used as the driver."
        );
    }
    // Build scripts (and also cargo: https://github.com/crablang/cargo/issues/10885) will invoke
    // `crablangc` even when `CRABLANGC_WRAPPER` is set. To make sure everything is coherent, we want that
    // to be the Miri driver, but acting as crablangc, on the target level. (Target, rather than host,
    // is needed for cross-interpretation situations.) This is not a perfect emulation of real crablangc
    // (it might be unable to produce binaries since the sysroot is check-only), but it's as close
    // as we can get, and it's good enough for autocfg.
    //
    // In `main`, we need the value of `CRABLANGC` to distinguish CRABLANGC_WRAPPER invocations from crablangdoc
    // or TARGET_RUNNER invocations, so we canonicalize it here to make it exceedingly unlikely that
    // there would be a collision with other invocations of cargo-miri (as crablangdoc or as runner). We
    // explicitly do this even if CRABLANGC_STAGE is set, since for these builds we do *not* want the
    // bootstrap `crablangc` thing in our way! Instead, we have MIRI_HOST_SYSROOT to use for host
    // builds.
    cmd.env("CRABLANGC", fs::canonicalize(find_miri()).unwrap());
    cmd.env("MIRI_BE_CRABLANGC", "target"); // we better remember to *unset* this in the other phases!

    // Set crablangdoc to us as well, so we can run doctests.
    cmd.env("CRABLANGDOC", &cargo_miri_path);

    cmd.env("MIRI_LOCAL_CRATES", local_crates(&metadata));
    if verbose > 0 {
        cmd.env("MIRI_VERBOSE", verbose.to_string()); // This makes the other phases verbose.
    }

    // Run cargo.
    debug_cmd("[cargo-miri miri]", verbose, &cmd);
    exec(cmd)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CrabLangcPhase {
    /// `crablangc` called during sysroot build.
    Setup,
    /// `crablangc` called by `cargo` for regular build.
    Build,
    /// `crablangc` called by `crablangdoc` for doctest.
    CrabLangdoc,
}

pub fn phase_crablangc(mut args: impl Iterator<Item = String>, phase: CrabLangcPhase) {
    /// Determines if we are being invoked (as crablangc) to build a crate for
    /// the "target" architecture, in contrast to the "host" architecture.
    /// Host crates are for build scripts and proc macros and still need to
    /// be built like normal; target crates need to be built for or interpreted
    /// by Miri.
    ///
    /// Currently, we detect this by checking for "--target=", which is
    /// never set for host crates. This matches what crablangc bootstrap does,
    /// which hopefully makes it "reliable enough". This relies on us always
    /// invoking cargo itself with `--target`, which `in_cargo_miri` ensures.
    fn is_target_crate() -> bool {
        get_arg_flag_value("--target").is_some()
    }

    /// Returns whether or not Cargo invoked the wrapper (this binary) to compile
    /// the final, binary crate (either a test for 'cargo test', or a binary for 'cargo run')
    /// Cargo does not give us this information directly, so we need to check
    /// various command-line flags.
    fn is_runnable_crate() -> bool {
        let is_bin = get_arg_flag_value("--crate-type").as_deref().unwrap_or("bin") == "bin";
        let is_test = has_arg_flag("--test");
        is_bin || is_test
    }

    fn out_filenames() -> Vec<PathBuf> {
        if let Some(out_file) = get_arg_flag_value("-o") {
            // `-o` has precedence over `--out-dir`.
            vec![PathBuf::from(out_file)]
        } else {
            let out_dir = get_arg_flag_value("--out-dir").unwrap_or_default();
            let path = PathBuf::from(out_dir);
            // Ask crablangc for the filename (since that is target-dependent).
            let mut crablangc = miri_for_host(); // sysroot doesn't matter for this so we just use the host
            crablangc.arg("--print").arg("file-names");
            for flag in ["--crate-name", "--crate-type", "--target"] {
                for val in get_arg_flag_values(flag) {
                    crablangc.arg(flag).arg(val);
                }
            }
            // This is technically passed as `-C extra-filename=...`, but the prefix seems unique
            // enough... (and cargo passes this before the filename so it should be unique)
            if let Some(extra) = get_arg_flag_value("extra-filename") {
                crablangc.arg("-C").arg(format!("extra-filename={extra}"));
            }
            crablangc.arg("-");

            let output = crablangc.output().expect("cannot run crablangc to determine file name");
            assert!(
                output.status.success(),
                "crablangc failed when determining file name:\n{output:?}"
            );
            let output =
                String::from_utf8(output.stdout).expect("crablangc returned non-UTF-8 filename");
            output.lines().filter(|l| !l.is_empty()).map(|l| path.join(l)).collect()
        }
    }

    // phase_cargo_miri set `MIRI_BE_CRABLANGC` for when build scripts directly invoke the driver;
    // however, if we get called back by cargo here, we'll carefully compute the right flags
    // ourselves, so we first un-do what the earlier phase did.
    env::remove_var("MIRI_BE_CRABLANGC");

    let verbose = std::env::var("MIRI_VERBOSE")
        .map_or(0, |verbose| verbose.parse().expect("verbosity flag must be an integer"));
    let target_crate = is_target_crate();
    // Determine whether this is cargo invoking crablangc to get some infos.
    let info_query = get_arg_flag_value("--print").is_some() || has_arg_flag("-vV");

    let store_json = |info: CrateRunInfo| {
        if get_arg_flag_value("--emit").unwrap_or_default().split(',').any(|e| e == "dep-info") {
            // Create a stub .d file to stop Cargo from "rebuilding" the crate:
            // https://github.com/crablang/miri/issues/1724#issuecomment-787115693
            // As we store a JSON file instead of building the crate here, an empty file is fine.
            let dep_info_name = format!(
                "{}/{}{}.d",
                get_arg_flag_value("--out-dir").unwrap(),
                get_arg_flag_value("--crate-name").unwrap(),
                get_arg_flag_value("extra-filename").unwrap_or_default(),
            );
            if verbose > 0 {
                eprintln!("[cargo-miri crablangc] writing stub dep-info to `{dep_info_name}`");
            }
            File::create(dep_info_name).expect("failed to create fake .d file");
        }

        for filename in out_filenames() {
            if verbose > 0 {
                eprintln!("[cargo-miri crablangc] writing run info to `{}`", filename.display());
            }
            info.store(&filename);
        }
    };

    let runnable_crate = !info_query && is_runnable_crate();

    if runnable_crate && target_crate {
        assert!(
            phase != CrabLangcPhase::Setup,
            "there should be no interpretation during sysroot build"
        );
        let inside_crablangdoc = phase == CrabLangcPhase::CrabLangdoc;
        // This is the binary or test crate that we want to interpret under Miri.
        // But we cannot run it here, as cargo invoked us as a compiler -- our stdin and stdout are not
        // like we want them.
        // Instead of compiling, we write JSON into the output file with all the relevant command-line flags
        // and environment variables; this is used when cargo calls us again in the CARGO_TARGET_RUNNER phase.
        let env = CrateRunEnv::collect(args, inside_crablangdoc);

        store_json(CrateRunInfo::RunWith(env.clone()));

        // CrabLangdoc expects us to exit with an error code if the test is marked as `compile_fail`,
        // just creating the JSON file is not enough: we need to detect syntax errors,
        // so we need to run Miri with `MIRI_BE_CRABLANGC` for a check-only build.
        if inside_crablangdoc {
            let mut cmd = miri();

            // Ensure --emit argument for a check-only build is present.
            if let Some(val) =
                ArgFlagValueIter::from_str_iter(env.args.iter().map(|s| s as &str), "--emit").next()
            {
                // For `no_run` tests, crablangdoc passes a `--emit` flag; make sure it has the right shape.
                assert_eq!(val, "metadata");
            } else {
                // For all other kinds of tests, we can just add our flag.
                cmd.arg("--emit=metadata");
            }

            // Alter the `-o` parameter so that it does not overwrite the JSON file we stored above.
            let mut args = env.args;
            let mut out_filename = None;
            for i in 0..args.len() {
                if args[i] == "-o" {
                    out_filename = Some(args[i + 1].clone());
                    args[i + 1].push_str(".miri");
                }
            }
            let out_filename = out_filename.expect("crablangdoc must pass `-o`");

            cmd.args(&args);
            cmd.env("MIRI_BE_CRABLANGC", "target");

            if verbose > 0 {
                eprintln!(
                    "[cargo-miri crablangc inside crablangdoc] captured input:\n{}",
                    std::str::from_utf8(&env.stdin).unwrap()
                );
                eprintln!("[cargo-miri crablangc inside crablangdoc] going to run:\n{cmd:?}");
            }

            exec_with_pipe(cmd, &env.stdin, format!("{out_filename}.stdin"));
        }

        return;
    }

    if runnable_crate && get_arg_flag_values("--extern").any(|krate| krate == "proc_macro") {
        // This is a "runnable" `proc-macro` crate (unit tests). We do not support
        // interpreting that under Miri now, so we write a JSON file to (display a
        // helpful message and) skip it in the runner phase.
        store_json(CrateRunInfo::SkipProcMacroTest);
        return;
    }

    let mut cmd = miri();
    let mut emit_link_hack = false;
    // Arguments are treated very differently depending on whether this crate is
    // for interpretation by Miri, or for use by a build script / proc macro.
    if !info_query && target_crate {
        // Forward arguments, but remove "link" from "--emit" to make this a check-only build.
        let emit_flag = "--emit";
        while let Some(arg) = args.next() {
            if let Some(val) = arg.strip_prefix(emit_flag) {
                // Patch this argument. First, extract its value.
                let val =
                    val.strip_prefix('=').expect("`cargo` should pass `--emit=X` as one argument");
                let mut val: Vec<_> = val.split(',').collect();
                // Now make sure "link" is not in there, but "metadata" is.
                if let Some(i) = val.iter().position(|&s| s == "link") {
                    emit_link_hack = true;
                    val.remove(i);
                    if !val.iter().any(|&s| s == "metadata") {
                        val.push("metadata");
                    }
                }
                cmd.arg(format!("{emit_flag}={}", val.join(",")));
            } else if arg == "--extern" {
                // Patch `--extern` filenames, since Cargo sometimes passes stub `.rlib` files:
                // https://github.com/crablang/miri/issues/1705
                forward_patched_extern_arg(&mut args, &mut cmd);
            } else {
                cmd.arg(arg);
            }
        }

        // During setup, patch the panic runtime for `libpanic_abort` (mirroring what bootstrap usually does).
        if phase == CrabLangcPhase::Setup
            && get_arg_flag_value("--crate-name").as_deref() == Some("panic_abort")
        {
            cmd.arg("-C").arg("panic=abort");
        }
    } else {
        // For host crates (but not when we are just printing some info),
        // we might still have to set the sysroot.
        if !info_query {
            // When we're running `cargo-miri` from `x.py` we need to pass the sysroot explicitly
            // due to bootstrap complications.
            if let Some(sysroot) = std::env::var_os("MIRI_HOST_SYSROOT") {
                cmd.arg("--sysroot").arg(sysroot);
            }
        }

        // For host crates or when we are printing, just forward everything.
        cmd.args(args);
    }

    // We want to compile, not interpret. We still use Miri to make sure the compiler version etc
    // are the exact same as what is used for interpretation.
    // MIRI_DEFAULT_ARGS should not be used to build host crates, hence setting "target" or "host"
    // as the value here to help Miri differentiate them.
    cmd.env("MIRI_BE_CRABLANGC", if target_crate { "target" } else { "host" });

    // Run it.
    if verbose > 0 {
        eprintln!(
            "[cargo-miri crablangc] target_crate={target_crate} runnable_crate={runnable_crate} info_query={info_query}"
        );
    }

    // Create a stub .rlib file if "link" was requested by cargo.
    // This is necessary to prevent cargo from doing rebuilds all the time.
    if emit_link_hack {
        for filename in out_filenames() {
            if verbose > 0 {
                eprintln!("[cargo-miri crablangc] creating fake lib file at `{}`", filename.display());
            }
            File::create(filename).expect("failed to create fake lib file");
        }
    }

    debug_cmd("[cargo-miri crablangc]", verbose, &cmd);
    exec(cmd);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RunnerPhase {
    /// `cargo` is running a binary
    Cargo,
    /// `crablangdoc` is running a binary
    CrabLangdoc,
}

pub fn phase_runner(mut binary_args: impl Iterator<Item = String>, phase: RunnerPhase) {
    // phase_cargo_miri set `MIRI_BE_CRABLANGC` for when build scripts directly invoke the driver;
    // however, if we get called back by cargo here, we'll carefully compute the right flags
    // ourselves, so we first un-do what the earlier phase did.
    env::remove_var("MIRI_BE_CRABLANGC");

    let verbose = std::env::var("MIRI_VERBOSE")
        .map_or(0, |verbose| verbose.parse().expect("verbosity flag must be an integer"));

    let binary = binary_args.next().unwrap();
    let file = File::open(&binary)
        .unwrap_or_else(|_| show_error!(
            "file {:?} not found or `cargo-miri` invoked incorrectly; please only invoke this binary through `cargo miri`", binary
        ));
    let file = BufReader::new(file);

    let info = serde_json::from_reader(file).unwrap_or_else(|_| {
        show_error!("file {:?} contains outdated or invalid JSON; try `cargo clean`", binary)
    });
    let info = match info {
        CrateRunInfo::RunWith(info) => info,
        CrateRunInfo::SkipProcMacroTest => {
            eprintln!(
                "Running unit tests of `proc-macro` crates is not currently supported by Miri."
            );
            return;
        }
    };

    let mut cmd = miri();

    // Set missing env vars. We prefer build-time env vars over run-time ones; see
    // <https://github.com/crablang/miri/issues/1661> for the kind of issue that fixes.
    for (name, val) in info.env {
        if let Some(old_val) = env::var_os(&name) {
            if old_val == val {
                // This one did not actually change, no need to re-set it.
                // (This keeps the `debug_cmd` below more manageable.)
                continue;
            } else if verbose > 0 {
                eprintln!(
                    "[cargo-miri runner] Overwriting run-time env var {name:?}={old_val:?} with build-time value {val:?}"
                );
            }
        }
        cmd.env(name, val);
    }

    // Forward crablangc arguments.
    // We need to patch "--extern" filenames because we forced a check-only
    // build without cargo knowing about that: replace `.rlib` suffix by
    // `.rmeta`.
    // We also need to remove `--error-format` as cargo specifies that to be JSON,
    // but when we run here, cargo does not interpret the JSON any more. `--json`
    // then also nees to be dropped.
    let mut args = info.args.into_iter();
    let error_format_flag = "--error-format";
    let json_flag = "--json";
    while let Some(arg) = args.next() {
        if arg == "--extern" {
            forward_patched_extern_arg(&mut args, &mut cmd);
        } else if let Some(suffix) = arg.strip_prefix(error_format_flag) {
            assert!(suffix.starts_with('='));
            // Drop this argument.
        } else if let Some(suffix) = arg.strip_prefix(json_flag) {
            assert!(suffix.starts_with('='));
            // Drop this argument.
        } else {
            cmd.arg(arg);
        }
    }
    // Respect `MIRIFLAGS`.
    if let Ok(a) = env::var("MIRIFLAGS") {
        // This code is taken from `CRABLANGFLAGS` handling in cargo.
        let args = a.split(' ').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        cmd.args(args);
    }

    // Then pass binary arguments.
    cmd.arg("--");
    cmd.args(binary_args);

    // Make sure we use the build-time working directory for interpreting Miri/crablangc arguments.
    // But then we need to switch to the run-time one, which we instruct Miri to do by setting `MIRI_CWD`.
    cmd.current_dir(info.current_dir);
    cmd.env("MIRI_CWD", env::current_dir().unwrap());

    // Run it.
    debug_cmd("[cargo-miri runner]", verbose, &cmd);
    match phase {
        RunnerPhase::CrabLangdoc => exec_with_pipe(cmd, &info.stdin, format!("{binary}.stdin")),
        RunnerPhase::Cargo => exec(cmd),
    }
}

pub fn phase_crablangdoc(mut args: impl Iterator<Item = String>) {
    let verbose = std::env::var("MIRI_VERBOSE")
        .map_or(0, |verbose| verbose.parse().expect("verbosity flag must be an integer"));

    // phase_cargo_miri sets the CRABLANGDOC env var to ourselves, so we can't use that here;
    // just default to a straight-forward invocation for now:
    let mut cmd = Command::new("crablangdoc");

    let extern_flag = "--extern";
    let runtool_flag = "--runtool";
    while let Some(arg) = args.next() {
        if arg == extern_flag {
            // Patch --extern arguments to use *.rmeta files, since phase_cargo_crablangc only creates stub *.rlib files.
            forward_patched_extern_arg(&mut args, &mut cmd);
        } else if arg == runtool_flag {
            // An existing --runtool flag indicates cargo is running in cross-target mode, which we don't support.
            // Note that this is only passed when cargo is run with the unstable -Zdoctest-xcompile flag;
            // otherwise, we won't be called as crablangdoc at all.
            show_error!("cross-interpreting doctests is not currently supported by Miri.");
        } else {
            cmd.arg(arg);
        }
    }

    // Doctests of `proc-macro` crates (and their dependencies) are always built for the host,
    // so we are not able to run them in Miri.
    if get_arg_flag_values("--crate-type").any(|crate_type| crate_type == "proc-macro") {
        eprintln!("Running doctests of `proc-macro` crates is not currently supported by Miri.");
        return;
    }

    // For each doctest, crablangdoc starts two child processes: first the test is compiled,
    // then the produced executable is invoked. We want to reroute both of these to cargo-miri,
    // such that the first time we'll enter phase_cargo_crablangc, and phase_cargo_runner second.
    //
    // crablangdoc invokes the test-builder by forwarding most of its own arguments, which makes
    // it difficult to determine when phase_cargo_crablangc should run instead of phase_cargo_crablangdoc.
    // Furthermore, the test code is passed via stdin, rather than a temporary file, so we need
    // to let phase_cargo_crablangc know to expect that. We'll use this environment variable as a flag:
    cmd.env("MIRI_CALLED_FROM_CRABLANGDOC", "1");

    // The `--test-builder` and `--runtool` arguments are unstable crablangdoc features,
    // which are disabled by default. We first need to enable them explicitly:
    cmd.arg("-Z").arg("unstable-options");

    // crablangdoc needs to know the right sysroot.
    cmd.arg("--sysroot").arg(env::var_os("MIRI_SYSROOT").unwrap());
    // make sure the 'miri' flag is set for crablangdoc
    cmd.arg("--cfg").arg("miri");

    // Make crablangdoc call us back.
    let cargo_miri_path = std::env::current_exe().expect("current executable path invalid");
    cmd.arg("--test-builder").arg(&cargo_miri_path); // invoked by forwarding most arguments
    cmd.arg("--runtool").arg(&cargo_miri_path); // invoked with just a single path argument

    debug_cmd("[cargo-miri crablangdoc]", verbose, &cmd);
    exec(cmd)
}
