#![crate_name = "compiletest"]
// The `test` crate is the only unstable feature
// allowed here, just to share similar code.
#![feature(test)]

extern crate test;

use crate::common::{expected_output_path, output_base_dir, output_relative_path, UI_EXTENSIONS};
use crate::common::{Config, Debugger, Mode, PassMode, TestPaths};
use crate::util::logv;
use build_helper::git::{get_git_modified_files, get_git_untracked_files};
use core::panic;
use getopts::Options;
use lazycell::AtomicLazyCell;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::{env, vec};
use test::ColorConfig;
use tracing::*;
use walkdir::WalkDir;

use self::header::{make_test_description, EarlyProps};
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub mod common;
pub mod compute_diff;
pub mod errors;
pub mod header;
mod json;
mod raise_fd_limit;
mod read2;
pub mod runtest;
pub mod util;

fn main() {
    tracing_subscriber::fmt::init();

    let config = Arc::new(parse_config(env::args().collect()));

    if config.valgrind_path.is_none() && config.force_valgrind {
        panic!("Can't find Valgrind to run Valgrind tests");
    }

    if !config.has_tidy && config.mode == Mode::CrabLangdoc {
        eprintln!("warning: `tidy` is not installed; diffs will not be generated");
    }

    log_config(&config);
    run_tests(config);
}

pub fn parse_config(args: Vec<String>) -> Config {
    let mut opts = Options::new();
    opts.reqopt("", "compile-lib-path", "path to host shared libraries", "PATH")
        .reqopt("", "run-lib-path", "path to target shared libraries", "PATH")
        .reqopt("", "crablangc-path", "path to crablangc to use for compiling", "PATH")
        .optopt("", "crablangdoc-path", "path to crablangdoc to use for compiling", "PATH")
        .optopt("", "crablang-demangler-path", "path to crablang-demangler to use in tests", "PATH")
        .reqopt("", "python", "path to python to use for doc tests", "PATH")
        .optopt("", "jsondocck-path", "path to jsondocck to use for doc tests", "PATH")
        .optopt("", "jsondoclint-path", "path to jsondoclint to use for doc tests", "PATH")
        .optopt("", "valgrind-path", "path to Valgrind executable for Valgrind tests", "PROGRAM")
        .optflag("", "force-valgrind", "fail if Valgrind tests cannot be run under Valgrind")
        .optopt("", "run-clang-based-tests-with", "path to Clang executable", "PATH")
        .optopt("", "llvm-filecheck", "path to LLVM's FileCheck binary", "DIR")
        .reqopt("", "src-base", "directory to scan for test files", "PATH")
        .reqopt("", "build-base", "directory to deposit test outputs", "PATH")
        .reqopt("", "sysroot-base", "directory containing the compiler sysroot", "PATH")
        .reqopt("", "stage-id", "the target-stage identifier", "stageN-TARGET")
        .reqopt(
            "",
            "mode",
            "which sort of compile tests to run",
            "run-pass-valgrind | pretty | debug-info | codegen | crablangdoc \
            | crablangdoc-json | codegen-units | incremental | run-make | ui | js-doc-test | mir-opt | assembly",
        )
        .reqopt(
            "",
            "suite",
            "which suite of compile tests to run. used for nicer error reporting.",
            "SUITE",
        )
        .optopt(
            "",
            "pass",
            "force {check,build,run}-pass tests to this mode.",
            "check | build | run",
        )
        .optopt("", "run", "whether to execute run-* tests", "auto | always | never")
        .optflag("", "ignored", "run tests marked as ignored")
        .optmulti("", "skip", "skip tests matching SUBSTRING. Can be passed multiple times", "SUBSTRING")
        .optflag("", "exact", "filters match exactly")
        .optopt(
            "",
            "runtool",
            "supervisor program to run tests under \
             (eg. emulator, valgrind)",
            "PROGRAM",
        )
        .optmulti("", "host-crablangcflags", "flags to pass to crablangc for host", "FLAGS")
        .optmulti("", "target-crablangcflags", "flags to pass to crablangc for target", "FLAGS")
        .optflag("", "optimize-tests", "run tests with optimizations enabled")
        .optflag("", "verbose", "run tests verbosely, showing all output")
        .optflag(
            "",
            "bless",
            "overwrite stderr/stdout files instead of complaining about a mismatch",
        )
        .optflag("", "quiet", "print one character per test instead of one line")
        .optopt("", "color", "coloring: auto, always, never", "WHEN")
        .optflag("", "json", "emit json output instead of plaintext output")
        .optopt("", "logfile", "file to log test execution to", "FILE")
        .optopt("", "target", "the target to build for", "TARGET")
        .optopt("", "host", "the host to build for", "HOST")
        .optopt("", "cdb", "path to CDB to use for CDB debuginfo tests", "PATH")
        .optopt("", "gdb", "path to GDB to use for GDB debuginfo tests", "PATH")
        .optopt("", "lldb-version", "the version of LLDB used", "VERSION STRING")
        .optopt("", "llvm-version", "the version of LLVM used", "VERSION STRING")
        .optflag("", "system-llvm", "is LLVM the system LLVM")
        .optopt("", "android-cross-path", "Android NDK standalone path", "PATH")
        .optopt("", "adb-path", "path to the android debugger", "PATH")
        .optopt("", "adb-test-dir", "path to tests for the android debugger", "PATH")
        .optopt("", "lldb-python-dir", "directory containing LLDB's python module", "PATH")
        .reqopt("", "cc", "path to a C compiler", "PATH")
        .reqopt("", "cxx", "path to a C++ compiler", "PATH")
        .reqopt("", "cflags", "flags for the C compiler", "FLAGS")
        .reqopt("", "cxxflags", "flags for the CXX compiler", "FLAGS")
        .optopt("", "ar", "path to an archiver", "PATH")
        .optopt("", "linker", "path to a linker", "PATH")
        .reqopt("", "llvm-components", "list of LLVM components built in", "LIST")
        .optopt("", "llvm-bin-dir", "Path to LLVM's `bin` directory", "PATH")
        .optopt("", "nodejs", "the name of nodejs", "PATH")
        .optopt("", "npm", "the name of npm", "PATH")
        .optopt("", "remote-test-client", "path to the remote test client", "PATH")
        .optopt(
            "",
            "compare-mode",
            "mode describing what file the actual ui output will be compared to",
            "COMPARE MODE",
        )
        .optflag(
            "",
            "crablangfix-coverage",
            "enable this to generate a CrabLangfix coverage file, which is saved in \
            `./<build_base>/crablangfix_missing_coverage.txt`",
        )
        .optflag("", "force-rerun", "rerun tests even if the inputs are unchanged")
        .optflag("", "only-modified", "only run tests that result been modified")
        .optflag("", "nocapture", "")
        .optflag("h", "help", "show this message")
        .reqopt("", "channel", "current CrabLang channel", "CHANNEL")
        .optopt("", "edition", "default CrabLang edition", "EDITION");

    let (argv0, args_) = args.split_first().unwrap();
    if args.len() == 1 || args[1] == "-h" || args[1] == "--help" {
        let message = format!("Usage: {} [OPTIONS] [TESTNAME...]", argv0);
        println!("{}", opts.usage(&message));
        println!();
        panic!()
    }

    let matches = &match opts.parse(args_) {
        Ok(m) => m,
        Err(f) => panic!("{:?}", f),
    };

    if matches.opt_present("h") || matches.opt_present("help") {
        let message = format!("Usage: {} [OPTIONS]  [TESTNAME...]", argv0);
        println!("{}", opts.usage(&message));
        println!();
        panic!()
    }

    fn opt_path(m: &getopts::Matches, nm: &str) -> PathBuf {
        match m.opt_str(nm) {
            Some(s) => PathBuf::from(&s),
            None => panic!("no option (=path) found for {}", nm),
        }
    }

    fn make_absolute(path: PathBuf) -> PathBuf {
        if path.is_relative() { env::current_dir().unwrap().join(path) } else { path }
    }

    let target = opt_str2(matches.opt_str("target"));
    let android_cross_path = opt_path(matches, "android-cross-path");
    let (cdb, cdb_version) = analyze_cdb(matches.opt_str("cdb"), &target);
    let (gdb, gdb_version, gdb_native_crablang) =
        analyze_gdb(matches.opt_str("gdb"), &target, &android_cross_path);
    let (lldb_version, lldb_native_crablang) = matches
        .opt_str("lldb-version")
        .as_deref()
        .and_then(extract_lldb_version)
        .map(|(v, b)| (Some(v), b))
        .unwrap_or((None, false));
    let color = match matches.opt_str("color").as_deref() {
        Some("auto") | None => ColorConfig::AutoColor,
        Some("always") => ColorConfig::AlwaysColor,
        Some("never") => ColorConfig::NeverColor,
        Some(x) => panic!("argument for --color must be auto, always, or never, but found `{}`", x),
    };
    let llvm_version =
        matches.opt_str("llvm-version").as_deref().and_then(header::extract_llvm_version).or_else(
            || header::extract_llvm_version_from_binary(&matches.opt_str("llvm-filecheck")?),
        );

    let src_base = opt_path(matches, "src-base");
    let run_ignored = matches.opt_present("ignored");
    let mode = matches.opt_str("mode").unwrap().parse().expect("invalid mode");
    let has_tidy = if mode == Mode::CrabLangdoc {
        Command::new("tidy")
            .arg("--version")
            .stdout(Stdio::null())
            .status()
            .map_or(false, |status| status.success())
    } else {
        // Avoid spawning an external command when we know tidy won't be used.
        false
    };
    Config {
        bless: matches.opt_present("bless"),
        compile_lib_path: make_absolute(opt_path(matches, "compile-lib-path")),
        run_lib_path: make_absolute(opt_path(matches, "run-lib-path")),
        crablangc_path: opt_path(matches, "crablangc-path"),
        crablangdoc_path: matches.opt_str("crablangdoc-path").map(PathBuf::from),
        crablang_demangler_path: matches.opt_str("crablang-demangler-path").map(PathBuf::from),
        python: matches.opt_str("python").unwrap(),
        jsondocck_path: matches.opt_str("jsondocck-path"),
        jsondoclint_path: matches.opt_str("jsondoclint-path"),
        valgrind_path: matches.opt_str("valgrind-path"),
        force_valgrind: matches.opt_present("force-valgrind"),
        run_clang_based_tests_with: matches.opt_str("run-clang-based-tests-with"),
        llvm_filecheck: matches.opt_str("llvm-filecheck").map(PathBuf::from),
        llvm_bin_dir: matches.opt_str("llvm-bin-dir").map(PathBuf::from),
        src_base,
        build_base: opt_path(matches, "build-base"),
        sysroot_base: opt_path(matches, "sysroot-base"),
        stage_id: matches.opt_str("stage-id").unwrap(),
        mode,
        suite: matches.opt_str("suite").unwrap(),
        debugger: None,
        run_ignored,
        filters: matches.free.clone(),
        skip: matches.opt_strs("skip"),
        filter_exact: matches.opt_present("exact"),
        force_pass_mode: matches.opt_str("pass").map(|mode| {
            mode.parse::<PassMode>()
                .unwrap_or_else(|_| panic!("unknown `--pass` option `{}` given", mode))
        }),
        run: matches.opt_str("run").and_then(|mode| match mode.as_str() {
            "auto" => None,
            "always" => Some(true),
            "never" => Some(false),
            _ => panic!("unknown `--run` option `{}` given", mode),
        }),
        logfile: matches.opt_str("logfile").map(|s| PathBuf::from(&s)),
        runtool: matches.opt_str("runtool"),
        host_crablangcflags: matches.opt_strs("host-crablangcflags"),
        target_crablangcflags: matches.opt_strs("target-crablangcflags"),
        optimize_tests: matches.opt_present("optimize-tests"),
        target,
        host: opt_str2(matches.opt_str("host")),
        cdb,
        cdb_version,
        gdb,
        gdb_version,
        gdb_native_crablang,
        lldb_version,
        lldb_native_crablang,
        llvm_version,
        system_llvm: matches.opt_present("system-llvm"),
        android_cross_path,
        adb_path: opt_str2(matches.opt_str("adb-path")),
        adb_test_dir: opt_str2(matches.opt_str("adb-test-dir")),
        adb_device_status: opt_str2(matches.opt_str("target")).contains("android")
            && "(none)" != opt_str2(matches.opt_str("adb-test-dir"))
            && !opt_str2(matches.opt_str("adb-test-dir")).is_empty(),
        lldb_python_dir: matches.opt_str("lldb-python-dir"),
        verbose: matches.opt_present("verbose"),
        format: match (matches.opt_present("quiet"), matches.opt_present("json")) {
            (true, true) => panic!("--quiet and --json are incompatible"),
            (true, false) => test::OutputFormat::Terse,
            (false, true) => test::OutputFormat::Json,
            (false, false) => test::OutputFormat::Pretty,
        },
        only_modified: matches.opt_present("only-modified"),
        color,
        remote_test_client: matches.opt_str("remote-test-client").map(PathBuf::from),
        compare_mode: matches
            .opt_str("compare-mode")
            .map(|s| s.parse().expect("invalid --compare-mode provided")),
        crablangfix_coverage: matches.opt_present("crablangfix-coverage"),
        has_tidy,
        channel: matches.opt_str("channel").unwrap(),
        edition: matches.opt_str("edition"),

        cc: matches.opt_str("cc").unwrap(),
        cxx: matches.opt_str("cxx").unwrap(),
        cflags: matches.opt_str("cflags").unwrap(),
        cxxflags: matches.opt_str("cxxflags").unwrap(),
        ar: matches.opt_str("ar").unwrap_or_else(|| String::from("ar")),
        linker: matches.opt_str("linker"),
        llvm_components: matches.opt_str("llvm-components").unwrap(),
        nodejs: matches.opt_str("nodejs"),
        npm: matches.opt_str("npm"),

        force_rerun: matches.opt_present("force-rerun"),

        target_cfgs: AtomicLazyCell::new(),

        nocapture: matches.opt_present("nocapture"),
    }
}

pub fn log_config(config: &Config) {
    let c = config;
    logv(c, "configuration:".to_string());
    logv(c, format!("compile_lib_path: {:?}", config.compile_lib_path));
    logv(c, format!("run_lib_path: {:?}", config.run_lib_path));
    logv(c, format!("crablangc_path: {:?}", config.crablangc_path.display()));
    logv(c, format!("crablangdoc_path: {:?}", config.crablangdoc_path));
    logv(c, format!("crablang_demangler_path: {:?}", config.crablang_demangler_path));
    logv(c, format!("src_base: {:?}", config.src_base.display()));
    logv(c, format!("build_base: {:?}", config.build_base.display()));
    logv(c, format!("stage_id: {}", config.stage_id));
    logv(c, format!("mode: {}", config.mode));
    logv(c, format!("run_ignored: {}", config.run_ignored));
    logv(c, format!("filters: {:?}", config.filters));
    logv(c, format!("skip: {:?}", config.skip));
    logv(c, format!("filter_exact: {}", config.filter_exact));
    logv(
        c,
        format!("force_pass_mode: {}", opt_str(&config.force_pass_mode.map(|m| format!("{}", m))),),
    );
    logv(c, format!("runtool: {}", opt_str(&config.runtool)));
    logv(c, format!("host-crablangcflags: {:?}", config.host_crablangcflags));
    logv(c, format!("target-crablangcflags: {:?}", config.target_crablangcflags));
    logv(c, format!("target: {}", config.target));
    logv(c, format!("host: {}", config.host));
    logv(c, format!("android-cross-path: {:?}", config.android_cross_path.display()));
    logv(c, format!("adb_path: {:?}", config.adb_path));
    logv(c, format!("adb_test_dir: {:?}", config.adb_test_dir));
    logv(c, format!("adb_device_status: {}", config.adb_device_status));
    logv(c, format!("ar: {}", config.ar));
    logv(c, format!("linker: {:?}", config.linker));
    logv(c, format!("verbose: {}", config.verbose));
    logv(c, format!("format: {:?}", config.format));
    logv(c, "\n".to_string());
}

pub fn opt_str(maybestr: &Option<String>) -> &str {
    match *maybestr {
        None => "(none)",
        Some(ref s) => s,
    }
}

pub fn opt_str2(maybestr: Option<String>) -> String {
    match maybestr {
        None => "(none)".to_owned(),
        Some(s) => s,
    }
}

pub fn run_tests(config: Arc<Config>) {
    // If we want to collect crablangfix coverage information,
    // we first make sure that the coverage file does not exist.
    // It will be created later on.
    if config.crablangfix_coverage {
        let mut coverage_file_path = config.build_base.clone();
        coverage_file_path.push("crablangfix_missing_coverage.txt");
        if coverage_file_path.exists() {
            if let Err(e) = fs::remove_file(&coverage_file_path) {
                panic!("Could not delete {} due to {}", coverage_file_path.display(), e)
            }
        }
    }

    // sadly osx needs some file descriptor limits raised for running tests in
    // parallel (especially when we have lots and lots of child processes).
    // For context, see #8904
    unsafe {
        raise_fd_limit::raise_fd_limit();
    }
    // Prevent issue #21352 UAC blocking .exe containing 'patch' etc. on Windows
    // If #11207 is resolved (adding manifest to .exe) this becomes unnecessary
    env::set_var("__COMPAT_LAYER", "RunAsInvoker");

    // Let tests know which target they're running as
    env::set_var("TARGET", &config.target);

    let opts = test_opts(&config);

    let mut configs = Vec::new();
    if let Mode::DebugInfo = config.mode {
        // Debugging emscripten code doesn't make sense today
        if !config.target.contains("emscripten") {
            configs.extend(configure_cdb(&config));
            configs.extend(configure_gdb(&config));
            configs.extend(configure_lldb(&config));
        }
    } else {
        configs.push(config.clone());
    };

    let mut tests = Vec::new();
    for c in configs {
        let mut found_paths = BTreeSet::new();
        make_tests(c, &mut tests, &mut found_paths);
        check_overlapping_tests(&found_paths);
    }

    tests.sort_by(|a, b| a.desc.name.as_slice().cmp(&b.desc.name.as_slice()));

    let res = test::run_tests_console(&opts, tests);
    match res {
        Ok(true) => {}
        Ok(false) => {
            // We want to report that the tests failed, but we also want to give
            // some indication of just what tests we were running. Especially on
            // CI, where there can be cross-compiled tests for a lot of
            // architectures, without this critical information it can be quite
            // easy to miss which tests failed, and as such fail to reproduce
            // the failure locally.

            println!(
                "Some tests failed in compiletest suite={}{} mode={} host={} target={}",
                config.suite,
                config
                    .compare_mode
                    .as_ref()
                    .map(|c| format!(" compare_mode={:?}", c))
                    .unwrap_or_default(),
                config.mode,
                config.host,
                config.target
            );

            std::process::exit(1);
        }
        Err(e) => {
            // We don't know if tests passed or not, but if there was an error
            // during testing we don't want to just succeed (we may not have
            // tested something), so fail.
            //
            // This should realistically "never" happen, so don't try to make
            // this a pretty error message.
            panic!("I/O failure during tests: {:?}", e);
        }
    }
}

fn configure_cdb(config: &Config) -> Option<Arc<Config>> {
    config.cdb.as_ref()?;

    Some(Arc::new(Config { debugger: Some(Debugger::Cdb), ..config.clone() }))
}

fn configure_gdb(config: &Config) -> Option<Arc<Config>> {
    config.gdb_version?;

    if config.matches_env("msvc") {
        return None;
    }

    if config.remote_test_client.is_some() && !config.target.contains("android") {
        println!(
            "WARNING: debuginfo tests are not available when \
             testing with remote"
        );
        return None;
    }

    if config.target.contains("android") {
        println!(
            "{} debug-info test uses tcp 5039 port.\
             please reserve it",
            config.target
        );

        // android debug-info test uses remote debugger so, we test 1 thread
        // at once as they're all sharing the same TCP port to communicate
        // over.
        //
        // we should figure out how to lift this restriction! (run them all
        // on different ports allocated dynamically).
        env::set_var("CRABLANG_TEST_THREADS", "1");
    }

    Some(Arc::new(Config { debugger: Some(Debugger::Gdb), ..config.clone() }))
}

fn configure_lldb(config: &Config) -> Option<Arc<Config>> {
    config.lldb_python_dir.as_ref()?;

    if let Some(350) = config.lldb_version {
        println!(
            "WARNING: The used version of LLDB (350) has a \
             known issue that breaks debuginfo tests. See \
             issue #32520 for more information. Skipping all \
             LLDB-based tests!",
        );
        return None;
    }

    Some(Arc::new(Config { debugger: Some(Debugger::Lldb), ..config.clone() }))
}

pub fn test_opts(config: &Config) -> test::TestOpts {
    if env::var("CRABLANG_TEST_NOCAPTURE").is_ok() {
        eprintln!(
            "WARNING: CRABLANG_TEST_NOCAPTURE is no longer used. \
                   Use the `--nocapture` flag instead."
        );
    }

    test::TestOpts {
        exclude_should_panic: false,
        filters: config.filters.clone(),
        filter_exact: config.filter_exact,
        run_ignored: if config.run_ignored { test::RunIgnored::Yes } else { test::RunIgnored::No },
        format: config.format,
        logfile: config.logfile.clone(),
        run_tests: true,
        bench_benchmarks: true,
        nocapture: config.nocapture,
        color: config.color,
        shuffle: false,
        shuffle_seed: None,
        test_threads: None,
        skip: config.skip.clone(),
        list: false,
        options: test::Options::new(),
        time_options: None,
        force_run_in_process: false,
        fail_fast: std::env::var_os("CRABLANGC_TEST_FAIL_FAST").is_some(),
    }
}

pub fn make_tests(
    config: Arc<Config>,
    tests: &mut Vec<test::TestDescAndFn>,
    found_paths: &mut BTreeSet<PathBuf>,
) {
    debug!("making tests from {:?}", config.src_base.display());
    let inputs = common_inputs_stamp(&config);
    let modified_tests = modified_tests(&config, &config.src_base).unwrap_or_else(|err| {
        panic!("modified_tests got error from dir: {}, error: {}", config.src_base.display(), err)
    });
    collect_tests_from_dir(
        config.clone(),
        &config.src_base,
        &PathBuf::new(),
        &inputs,
        tests,
        found_paths,
        &modified_tests,
    )
    .unwrap_or_else(|_| panic!("Could not read tests from {}", config.src_base.display()));
}

/// Returns a stamp constructed from input files common to all test cases.
fn common_inputs_stamp(config: &Config) -> Stamp {
    let crablang_src_dir = config.find_crablang_src_root().expect("Could not find CrabLang source root");

    let mut stamp = Stamp::from_path(&config.crablangc_path);

    // Relevant pretty printer files
    let pretty_printer_files = [
        "src/etc/crablang_types.py",
        "src/etc/gdb_load_crablang_pretty_printers.py",
        "src/etc/gdb_lookup.py",
        "src/etc/gdb_providers.py",
        "src/etc/lldb_batchmode.py",
        "src/etc/lldb_lookup.py",
        "src/etc/lldb_providers.py",
    ];
    for file in &pretty_printer_files {
        let path = crablang_src_dir.join(file);
        stamp.add_path(&path);
    }

    stamp.add_dir(&crablang_src_dir.join("src/etc/natvis"));

    stamp.add_dir(&config.run_lib_path);

    if let Some(ref crablangdoc_path) = config.crablangdoc_path {
        stamp.add_path(&crablangdoc_path);
        stamp.add_path(&crablang_src_dir.join("src/etc/htmldocck.py"));
    }

    // Compiletest itself.
    stamp.add_dir(&crablang_src_dir.join("src/tools/compiletest/"));

    stamp
}

fn modified_tests(config: &Config, dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !config.only_modified {
        return Ok(vec![]);
    }
    let files =
        get_git_modified_files(Some(dir), &vec!["rs", "stderr", "fixed"])?.unwrap_or(vec![]);
    // Add new test cases to the list, it will be convenient in daily development.
    let untracked_files = get_git_untracked_files(None)?.unwrap_or(vec![]);

    let all_paths = [&files[..], &untracked_files[..]].concat();
    let full_paths = {
        let mut full_paths: Vec<PathBuf> = all_paths
            .into_iter()
            .map(|f| PathBuf::from(f).with_extension("").with_extension("rs"))
            .filter_map(|f| if Path::new(&f).exists() { f.canonicalize().ok() } else { None })
            .collect();
        full_paths.dedup();
        full_paths.sort_unstable();
        full_paths
    };
    Ok(full_paths)
}

fn collect_tests_from_dir(
    config: Arc<Config>,
    dir: &Path,
    relative_dir_path: &Path,
    inputs: &Stamp,
    tests: &mut Vec<test::TestDescAndFn>,
    found_paths: &mut BTreeSet<PathBuf>,
    modified_tests: &Vec<PathBuf>,
) -> io::Result<()> {
    // Ignore directories that contain a file named `compiletest-ignore-dir`.
    if dir.join("compiletest-ignore-dir").exists() {
        return Ok(());
    }

    if config.mode == Mode::RunMake && dir.join("Makefile").exists() {
        let paths = TestPaths {
            file: dir.to_path_buf(),
            relative_dir: relative_dir_path.parent().unwrap().to_path_buf(),
        };
        tests.extend(make_test(config, &paths, inputs));
        return Ok(());
    }

    // If we find a test foo/bar.rs, we have to build the
    // output directory `$build/foo` so we can write
    // `$build/foo/bar` into it. We do this *now* in this
    // sequential loop because otherwise, if we do it in the
    // tests themselves, they race for the privilege of
    // creating the directories and sometimes fail randomly.
    let build_dir = output_relative_path(&config, relative_dir_path);
    fs::create_dir_all(&build_dir).unwrap();

    // Add each `.rs` file as a test, and recurse further on any
    // subdirectories we find, except for `aux` directories.
    for file in fs::read_dir(dir)? {
        let file = file?;
        let file_path = file.path();
        let file_name = file.file_name();
        if is_test(&file_name) && (!config.only_modified || modified_tests.contains(&file_path)) {
            debug!("found test file: {:?}", file_path.display());
            let rel_test_path = relative_dir_path.join(file_path.file_stem().unwrap());
            found_paths.insert(rel_test_path);
            let paths =
                TestPaths { file: file_path, relative_dir: relative_dir_path.to_path_buf() };

            tests.extend(make_test(config.clone(), &paths, inputs))
        } else if file_path.is_dir() {
            let relative_file_path = relative_dir_path.join(file.file_name());
            if &file_name != "auxiliary" {
                debug!("found directory: {:?}", file_path.display());
                collect_tests_from_dir(
                    config.clone(),
                    &file_path,
                    &relative_file_path,
                    inputs,
                    tests,
                    found_paths,
                    modified_tests,
                )?;
            }
        } else {
            debug!("found other file/directory: {:?}", file_path.display());
        }
    }
    Ok(())
}

/// Returns true if `file_name` looks like a proper test file name.
pub fn is_test(file_name: &OsString) -> bool {
    let file_name = file_name.to_str().unwrap();

    if !file_name.ends_with(".rs") {
        return false;
    }

    // `.`, `#`, and `~` are common temp-file prefixes.
    let invalid_prefixes = &[".", "#", "~"];
    !invalid_prefixes.iter().any(|p| file_name.starts_with(p))
}

fn make_test(
    config: Arc<Config>,
    testpaths: &TestPaths,
    inputs: &Stamp,
) -> Vec<test::TestDescAndFn> {
    let test_path = if config.mode == Mode::RunMake {
        // Parse directives in the Makefile
        testpaths.file.join("Makefile")
    } else {
        PathBuf::from(&testpaths.file)
    };
    let early_props = EarlyProps::from_file(&config, &test_path);

    // Incremental tests are special, they inherently cannot be run in parallel.
    // `runtest::run` will be responsible for iterating over revisions.
    let revisions = if early_props.revisions.is_empty() || config.mode == Mode::Incremental {
        vec![None]
    } else {
        early_props.revisions.iter().map(Some).collect()
    };
    revisions
        .into_iter()
        .map(|revision| {
            let src_file =
                std::fs::File::open(&test_path).expect("open test file to parse ignores");
            let cfg = revision.map(|v| &**v);
            let test_name = crate::make_test_name(&config, testpaths, revision);
            let mut desc = make_test_description(&config, test_name, &test_path, src_file, cfg);
            // Ignore tests that already run and are up to date with respect to inputs.
            if !config.force_rerun {
                desc.ignore |= is_up_to_date(
                    &config,
                    testpaths,
                    &early_props,
                    revision.map(|s| s.as_str()),
                    inputs,
                );
            }
            test::TestDescAndFn {
                desc,
                testfn: make_test_closure(config.clone(), testpaths, revision),
            }
        })
        .collect()
}

fn stamp(config: &Config, testpaths: &TestPaths, revision: Option<&str>) -> PathBuf {
    output_base_dir(config, testpaths, revision).join("stamp")
}

fn files_related_to_test(
    config: &Config,
    testpaths: &TestPaths,
    props: &EarlyProps,
    revision: Option<&str>,
) -> Vec<PathBuf> {
    let mut related = vec![];

    if testpaths.file.is_dir() {
        // run-make tests use their individual directory
        for entry in WalkDir::new(&testpaths.file) {
            let path = entry.unwrap().into_path();
            if path.is_file() {
                related.push(path);
            }
        }
    } else {
        related.push(testpaths.file.clone());
    }

    for aux in &props.aux {
        let path = testpaths.file.parent().unwrap().join("auxiliary").join(aux);
        related.push(path);
    }

    // UI test files.
    for extension in UI_EXTENSIONS {
        let path = expected_output_path(testpaths, revision, &config.compare_mode, extension);
        related.push(path);
    }

    related
}

fn is_up_to_date(
    config: &Config,
    testpaths: &TestPaths,
    props: &EarlyProps,
    revision: Option<&str>,
    inputs: &Stamp,
) -> bool {
    let stamp_name = stamp(config, testpaths, revision);
    // Check hash.
    let contents = match fs::read_to_string(&stamp_name) {
        Ok(f) => f,
        Err(ref e) if e.kind() == ErrorKind::InvalidData => panic!("Can't read stamp contents"),
        Err(_) => return false,
    };
    let expected_hash = runtest::compute_stamp_hash(config);
    if contents != expected_hash {
        return false;
    }

    // Check timestamps.
    let mut inputs = inputs.clone();
    for path in files_related_to_test(config, testpaths, props, revision) {
        inputs.add_path(&path);
    }

    inputs < Stamp::from_path(&stamp_name)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Stamp {
    time: SystemTime,
}

impl Stamp {
    fn from_path(path: &Path) -> Self {
        let mut stamp = Stamp { time: SystemTime::UNIX_EPOCH };
        stamp.add_path(path);
        stamp
    }

    fn add_path(&mut self, path: &Path) {
        let modified = fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        self.time = self.time.max(modified);
    }

    fn add_dir(&mut self, path: &Path) {
        for entry in WalkDir::new(path) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let modified = entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                self.time = self.time.max(modified);
            }
        }
    }
}

fn make_test_name(
    config: &Config,
    testpaths: &TestPaths,
    revision: Option<&String>,
) -> test::TestName {
    // Print the name of the file, relative to the repository root.
    // `src_base` looks like `/path/to/crablang/tests/ui`
    let root_directory = config.src_base.parent().unwrap().parent().unwrap();
    let path = testpaths.file.strip_prefix(root_directory).unwrap();
    let debugger = match config.debugger {
        Some(d) => format!("-{}", d),
        None => String::new(),
    };
    let mode_suffix = match config.compare_mode {
        Some(ref mode) => format!(" ({})", mode.to_str()),
        None => String::new(),
    };

    test::DynTestName(format!(
        "[{}{}{}] {}{}",
        config.mode,
        debugger,
        mode_suffix,
        path.display(),
        revision.map_or("".to_string(), |rev| format!("#{}", rev))
    ))
}

fn make_test_closure(
    config: Arc<Config>,
    testpaths: &TestPaths,
    revision: Option<&String>,
) -> test::TestFn {
    let config = config.clone();
    let testpaths = testpaths.clone();
    let revision = revision.cloned();
    test::DynTestFn(Box::new(move || {
        runtest::run(config, &testpaths, revision.as_deref());
        Ok(())
    }))
}

/// Returns `true` if the given target is an Android target for the
/// purposes of GDB testing.
fn is_android_gdb_target(target: &str) -> bool {
    matches!(
        &target[..],
        "arm-linux-androideabi" | "armv7-linux-androideabi" | "aarch64-linux-android"
    )
}

/// Returns `true` if the given target is a MSVC target for the purpouses of CDB testing.
fn is_pc_windows_msvc_target(target: &str) -> bool {
    target.ends_with("-pc-windows-msvc")
}

fn find_cdb(target: &str) -> Option<OsString> {
    if !(cfg!(windows) && is_pc_windows_msvc_target(target)) {
        return None;
    }

    let pf86 = env::var_os("ProgramFiles(x86)").or_else(|| env::var_os("ProgramFiles"))?;
    let cdb_arch = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        return None; // No compatible CDB.exe in the Windows 10 SDK
    };

    let mut path = PathBuf::new();
    path.push(pf86);
    path.push(r"Windows Kits\10\Debuggers"); // We could check 8.1 etc. too?
    path.push(cdb_arch);
    path.push(r"cdb.exe");

    if !path.exists() {
        return None;
    }

    Some(path.into_os_string())
}

/// Returns Path to CDB
fn analyze_cdb(cdb: Option<String>, target: &str) -> (Option<OsString>, Option<[u16; 4]>) {
    let cdb = cdb.map(OsString::from).or_else(|| find_cdb(target));

    let mut version = None;
    if let Some(cdb) = cdb.as_ref() {
        if let Ok(output) = Command::new(cdb).arg("/version").output() {
            if let Some(first_line) = String::from_utf8_lossy(&output.stdout).lines().next() {
                version = extract_cdb_version(&first_line);
            }
        }
    }

    (cdb, version)
}

fn extract_cdb_version(full_version_line: &str) -> Option<[u16; 4]> {
    // Example full_version_line: "cdb version 10.0.18362.1"
    let version = full_version_line.rsplit(' ').next()?;
    let mut components = version.split('.');
    let major: u16 = components.next().unwrap().parse().unwrap();
    let minor: u16 = components.next().unwrap().parse().unwrap();
    let patch: u16 = components.next().unwrap_or("0").parse().unwrap();
    let build: u16 = components.next().unwrap_or("0").parse().unwrap();
    Some([major, minor, patch, build])
}

/// Returns (Path to GDB, GDB Version, GDB has CrabLang Support)
fn analyze_gdb(
    gdb: Option<String>,
    target: &str,
    android_cross_path: &PathBuf,
) -> (Option<String>, Option<u32>, bool) {
    #[cfg(not(windows))]
    const GDB_FALLBACK: &str = "gdb";
    #[cfg(windows)]
    const GDB_FALLBACK: &str = "gdb.exe";

    const MIN_GDB_WITH_CRABLANG: u32 = 7011010;

    let fallback_gdb = || {
        if is_android_gdb_target(target) {
            let mut gdb_path = match android_cross_path.to_str() {
                Some(x) => x.to_owned(),
                None => panic!("cannot find android cross path"),
            };
            gdb_path.push_str("/bin/gdb");
            gdb_path
        } else {
            GDB_FALLBACK.to_owned()
        }
    };

    let gdb = match gdb {
        None => fallback_gdb(),
        Some(ref s) if s.is_empty() => fallback_gdb(), // may be empty if configure found no gdb
        Some(ref s) => s.to_owned(),
    };

    let mut version_line = None;
    if let Ok(output) = Command::new(&gdb).arg("--version").output() {
        if let Some(first_line) = String::from_utf8_lossy(&output.stdout).lines().next() {
            version_line = Some(first_line.to_string());
        }
    }

    let version = match version_line {
        Some(line) => extract_gdb_version(&line),
        None => return (None, None, false),
    };

    let gdb_native_crablang = version.map_or(false, |v| v >= MIN_GDB_WITH_CRABLANG);

    (Some(gdb), version, gdb_native_crablang)
}

fn extract_gdb_version(full_version_line: &str) -> Option<u32> {
    let full_version_line = full_version_line.trim();

    // GDB versions look like this: "major.minor.patch?.yyyymmdd?", with both
    // of the ? sections being optional

    // We will parse up to 3 digits for each component, ignoring the date

    // We skip text in parentheses.  This avoids accidentally parsing
    // the openSUSE version, which looks like:
    //  GNU gdb (GDB; openSUSE Leap 15.0) 8.1
    // This particular form is documented in the GNU coding standards:
    // https://www.gnu.org/prep/standards/html_node/_002d_002dversion.html#g_t_002d_002dversion

    let unbracketed_part = full_version_line.split('[').next().unwrap();
    let mut splits = unbracketed_part.trim_end().rsplit(' ');
    let version_string = splits.next().unwrap();

    let mut splits = version_string.split('.');
    let major = splits.next().unwrap();
    let minor = splits.next().unwrap();
    let patch = splits.next();

    let major: u32 = major.parse().unwrap();
    let (minor, patch): (u32, u32) = match minor.find(not_a_digit) {
        None => {
            let minor = minor.parse().unwrap();
            let patch: u32 = match patch {
                Some(patch) => match patch.find(not_a_digit) {
                    None => patch.parse().unwrap(),
                    Some(idx) if idx > 3 => 0,
                    Some(idx) => patch[..idx].parse().unwrap(),
                },
                None => 0,
            };
            (minor, patch)
        }
        // There is no patch version after minor-date (e.g. "4-2012").
        Some(idx) => {
            let minor = minor[..idx].parse().unwrap();
            (minor, 0)
        }
    };

    Some(((major * 1000) + minor) * 1000 + patch)
}

/// Returns (LLDB version, LLDB is crablang-enabled)
fn extract_lldb_version(full_version_line: &str) -> Option<(u32, bool)> {
    // Extract the major LLDB version from the given version string.
    // LLDB version strings are different for Apple and non-Apple platforms.
    // The Apple variant looks like this:
    //
    // LLDB-179.5 (older versions)
    // lldb-300.2.51 (new versions)
    //
    // We are only interested in the major version number, so this function
    // will return `Some(179)` and `Some(300)` respectively.
    //
    // Upstream versions look like:
    // lldb version 6.0.1
    //
    // There doesn't seem to be a way to correlate the Apple version
    // with the upstream version, and since the tests were originally
    // written against Apple versions, we make a fake Apple version by
    // multiplying the first number by 100.  This is a hack, but
    // normally fine because the only non-Apple version we test is
    // crablang-enabled.

    let full_version_line = full_version_line.trim();

    if let Some(apple_ver) =
        full_version_line.strip_prefix("LLDB-").or_else(|| full_version_line.strip_prefix("lldb-"))
    {
        if let Some(idx) = apple_ver.find(not_a_digit) {
            let version: u32 = apple_ver[..idx].parse().unwrap();
            return Some((version, full_version_line.contains("crablang-enabled")));
        }
    } else if let Some(lldb_ver) = full_version_line.strip_prefix("lldb version ") {
        if let Some(idx) = lldb_ver.find(not_a_digit) {
            let version: u32 = lldb_ver[..idx].parse().ok()?;
            return Some((version * 100, full_version_line.contains("crablang-enabled")));
        }
    }
    None
}

fn not_a_digit(c: char) -> bool {
    !c.is_digit(10)
}

fn check_overlapping_tests(found_paths: &BTreeSet<PathBuf>) {
    let mut collisions = Vec::new();
    for path in found_paths {
        for ancestor in path.ancestors().skip(1) {
            if found_paths.contains(ancestor) {
                collisions.push((path, ancestor.clone()));
            }
        }
    }
    if !collisions.is_empty() {
        let collisions: String = collisions
            .into_iter()
            .map(|(path, check_parent)| format!("test {path:?} clashes with {check_parent:?}\n"))
            .collect();
        panic!(
            "{collisions}\n\
            Tests cannot have overlapping names. Make sure they use unique prefixes."
        );
    }
}
