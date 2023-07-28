// ignore-tidy-filelength

use crate::common::{expected_output_path, UI_EXTENSIONS, UI_FIXED, UI_STDERR, UI_STDOUT};
use crate::common::{incremental_dir, output_base_dir, output_base_name, output_testname_unique};
use crate::common::{Assembly, Incremental, JsDocTest, MirOpt, RunMake, RustdocJson, Ui};
use crate::common::{Codegen, CodegenUnits, DebugInfo, Debugger, Rustdoc};
use crate::common::{CompareMode, FailMode, PassMode};
use crate::common::{Config, TestPaths};
use crate::common::{Pretty, RunCoverage, RunPassValgrind};
use crate::common::{UI_COVERAGE, UI_RUN_STDERR, UI_RUN_STDOUT};
use crate::compute_diff::{write_diff, write_filtered_diff};
use crate::errors::{self, Error, ErrorKind};
use crate::header::TestProps;
use crate::json;
use crate::read2::read2_abbreviated;
use crate::util::{add_dylib_path, dylib_env_var, logv, PathBufExt};
use crate::ColorConfig;
use regex::{Captures, Regex};
use rustfix::{apply_suggestions, get_suggestions_from_json, Filter};

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, create_dir_all, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::iter;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::str;
use std::sync::Arc;

use anyhow::Context;
use glob::glob;
use once_cell::sync::Lazy;
use tracing::*;

use crate::extract_gdb_version;
use crate::is_android_gdb_target;

mod debugger;
use debugger::DebuggerCommands;

#[cfg(test)]
mod tests;

const FAKE_SRC_BASE: &str = "fake-test-src-base";

#[cfg(windows)]
fn disable_error_reporting<F: FnOnce() -> R, R>(f: F) -> R {
    use std::sync::Mutex;

    use windows::Win32::System::Diagnostics::Debug::{
        SetErrorMode, SEM_NOGPFAULTERRORBOX, THREAD_ERROR_MODE,
    };

    static LOCK: Mutex<()> = Mutex::new(());

    // Error mode is a global variable, so lock it so only one thread will change it
    let _lock = LOCK.lock().unwrap();

    // Tell Windows to not show any UI on errors (such as terminating abnormally).
    // This is important for running tests, since some of them use abnormal
    // termination by design. This mode is inherited by all child processes.
    unsafe {
        let old_mode = SetErrorMode(SEM_NOGPFAULTERRORBOX); // read inherited flags
        let old_mode = THREAD_ERROR_MODE(old_mode);
        SetErrorMode(old_mode | SEM_NOGPFAULTERRORBOX);
        let r = f();
        SetErrorMode(old_mode);
        r
    }
}

#[cfg(not(windows))]
fn disable_error_reporting<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

/// The platform-specific library name
pub fn get_lib_name(lib: &str, dylib: bool) -> String {
    // In some casess (e.g. MUSL), we build a static
    // library, rather than a dynamic library.
    // In this case, the only path we can pass
    // with '--extern-meta' is the '.lib' file
    if !dylib {
        return format!("lib{}.rlib", lib);
    }

    if cfg!(windows) {
        format!("{}.dll", lib)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", lib)
    } else {
        format!("lib{}.so", lib)
    }
}

pub fn run(config: Arc<Config>, testpaths: &TestPaths, revision: Option<&str>) {
    match &*config.target {
        "arm-linux-androideabi"
        | "armv7-linux-androideabi"
        | "thumbv7neon-linux-androideabi"
        | "aarch64-linux-android" => {
            if !config.adb_device_status {
                panic!("android device not available");
            }
        }

        _ => {
            // android has its own gdb handling
            if config.debugger == Some(Debugger::Gdb) && config.gdb.is_none() {
                panic!("gdb not available but debuginfo gdb debuginfo test requested");
            }
        }
    }

    if config.verbose {
        // We're going to be dumping a lot of info. Start on a new line.
        print!("\n\n");
    }
    debug!("running {:?}", testpaths.file.display());
    let mut props = TestProps::from_file(&testpaths.file, revision, &config);

    // For non-incremental (i.e. regular UI) tests, the incremental directory
    // takes into account the revision name, since the revisions are independent
    // of each other and can race.
    if props.incremental {
        props.incremental_dir = Some(incremental_dir(&config, testpaths, revision));
    }

    let cx = TestCx { config: &config, props: &props, testpaths, revision };
    create_dir_all(&cx.output_base_dir())
        .with_context(|| {
            format!("failed to create output base directory {}", cx.output_base_dir().display())
        })
        .unwrap();
    if props.incremental {
        cx.init_incremental_test();
    }

    if config.mode == Incremental {
        // Incremental tests are special because they cannot be run in
        // parallel.
        assert!(!props.revisions.is_empty(), "Incremental tests require revisions.");
        for revision in &props.revisions {
            let mut revision_props = TestProps::from_file(&testpaths.file, Some(revision), &config);
            revision_props.incremental_dir = props.incremental_dir.clone();
            let rev_cx = TestCx {
                config: &config,
                props: &revision_props,
                testpaths,
                revision: Some(revision),
            };
            rev_cx.run_revision();
        }
    } else {
        cx.run_revision();
    }

    cx.create_stamp();
}

pub fn compute_stamp_hash(config: &Config) -> String {
    let mut hash = DefaultHasher::new();
    config.stage_id.hash(&mut hash);
    config.run.hash(&mut hash);

    match config.debugger {
        Some(Debugger::Cdb) => {
            config.cdb.hash(&mut hash);
        }

        Some(Debugger::Gdb) => {
            config.gdb.hash(&mut hash);
            env::var_os("PATH").hash(&mut hash);
            env::var_os("PYTHONPATH").hash(&mut hash);
        }

        Some(Debugger::Lldb) => {
            config.python.hash(&mut hash);
            config.lldb_python_dir.hash(&mut hash);
            env::var_os("PATH").hash(&mut hash);
            env::var_os("PYTHONPATH").hash(&mut hash);
        }

        None => {}
    }

    if let Ui = config.mode {
        config.force_pass_mode.hash(&mut hash);
    }

    format!("{:x}", hash.finish())
}

#[derive(Copy, Clone)]
struct TestCx<'test> {
    config: &'test Config,
    props: &'test TestProps,
    testpaths: &'test TestPaths,
    revision: Option<&'test str>,
}

enum ReadFrom {
    Path,
    Stdin(String),
}

enum TestOutput {
    Compile,
    Run,
}

/// Will this test be executed? Should we use `make_exe_name`?
#[derive(Copy, Clone, PartialEq)]
enum WillExecute {
    Yes,
    No,
    Disabled,
}

/// What value should be passed to `--emit`?
#[derive(Copy, Clone)]
enum Emit {
    None,
    Metadata,
    LlvmIr,
    Asm,
    LinkArgsAsm,
}

impl<'test> TestCx<'test> {
    /// Code executed for each revision in turn (or, if there are no
    /// revisions, exactly once, with revision == None).
    fn run_revision(&self) {
        if self.props.should_ice && self.config.mode != Incremental {
            self.fatal("cannot use should-ice in a test that is not cfail");
        }
        match self.config.mode {
            RunPassValgrind => self.run_valgrind_test(),
            Pretty => self.run_pretty_test(),
            DebugInfo => self.run_debuginfo_test(),
            Codegen => self.run_codegen_test(),
            Rustdoc => self.run_rustdoc_test(),
            RustdocJson => self.run_rustdoc_json_test(),
            CodegenUnits => self.run_codegen_units_test(),
            Incremental => self.run_incremental_test(),
            RunMake => self.run_rmake_test(),
            Ui => self.run_ui_test(),
            MirOpt => self.run_mir_opt_test(),
            Assembly => self.run_assembly_test(),
            JsDocTest => self.run_js_doc_test(),
            RunCoverage => self.run_coverage_test(),
        }
    }

    fn pass_mode(&self) -> Option<PassMode> {
        self.props.pass_mode(self.config)
    }

    fn should_run(&self, pm: Option<PassMode>) -> WillExecute {
        let test_should_run = match self.config.mode {
            Ui if pm == Some(PassMode::Run) || self.props.fail_mode == Some(FailMode::Run) => true,
            MirOpt if pm == Some(PassMode::Run) => true,
            Ui | MirOpt => false,
            mode => panic!("unimplemented for mode {:?}", mode),
        };
        if test_should_run { self.run_if_enabled() } else { WillExecute::No }
    }

    fn run_if_enabled(&self) -> WillExecute {
        if self.config.run_enabled() { WillExecute::Yes } else { WillExecute::Disabled }
    }

    fn should_run_successfully(&self, pm: Option<PassMode>) -> bool {
        match self.config.mode {
            Ui | MirOpt => pm == Some(PassMode::Run),
            mode => panic!("unimplemented for mode {:?}", mode),
        }
    }

    fn should_compile_successfully(&self, pm: Option<PassMode>) -> bool {
        match self.config.mode {
            JsDocTest => true,
            Ui => pm.is_some() || self.props.fail_mode > Some(FailMode::Build),
            Incremental => {
                let revision =
                    self.revision.expect("incremental tests require a list of revisions");
                if revision.starts_with("cpass")
                    || revision.starts_with("rpass")
                    || revision.starts_with("rfail")
                {
                    true
                } else if revision.starts_with("cfail") {
                    pm.is_some()
                } else {
                    panic!("revision name must begin with cpass, rpass, rfail, or cfail");
                }
            }
            mode => panic!("unimplemented for mode {:?}", mode),
        }
    }

    fn check_if_test_should_compile(&self, proc_res: &ProcRes, pm: Option<PassMode>) {
        if self.should_compile_successfully(pm) {
            if !proc_res.status.success() {
                self.fatal_proc_rec("test compilation failed although it shouldn't!", proc_res);
            }
        } else {
            if proc_res.status.success() {
                self.fatal_proc_rec(
                    &format!("{} test compiled successfully!", self.config.mode)[..],
                    proc_res,
                );
            }

            if !self.props.dont_check_failure_status {
                self.check_correct_failure_status(proc_res);
            }
        }
    }

    fn run_cfail_test(&self) {
        let pm = self.pass_mode();
        let proc_res = self.compile_test(WillExecute::No, self.should_emit_metadata(pm));
        self.check_if_test_should_compile(&proc_res, pm);
        self.check_no_compiler_crash(&proc_res, self.props.should_ice);

        let output_to_check = self.get_output(&proc_res);
        let expected_errors = errors::load_errors(&self.testpaths.file, self.revision);
        if !expected_errors.is_empty() {
            if !self.props.error_patterns.is_empty() || !self.props.regex_error_patterns.is_empty()
            {
                self.fatal("both error pattern and expected errors specified");
            }
            self.check_expected_errors(expected_errors, &proc_res);
        } else {
            self.check_all_error_patterns(&output_to_check, &proc_res, pm);
        }
        if self.props.should_ice {
            match proc_res.status.code() {
                Some(101) => (),
                _ => self.fatal("expected ICE"),
            }
        }

        self.check_forbid_output(&output_to_check, &proc_res);
    }

    fn run_rfail_test(&self) {
        let pm = self.pass_mode();
        let should_run = self.run_if_enabled();
        let proc_res = self.compile_test(should_run, self.should_emit_metadata(pm));

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        if let WillExecute::Disabled = should_run {
            return;
        }

        let proc_res = self.exec_compiled_test();

        // The value our Makefile configures valgrind to return on failure
        const VALGRIND_ERR: i32 = 100;
        if proc_res.status.code() == Some(VALGRIND_ERR) {
            self.fatal_proc_rec("run-fail test isn't valgrind-clean!", &proc_res);
        }

        let output_to_check = self.get_output(&proc_res);
        self.check_correct_failure_status(&proc_res);
        self.check_all_error_patterns(&output_to_check, &proc_res, pm);
    }

    fn get_output(&self, proc_res: &ProcRes) -> String {
        if self.props.check_stdout {
            format!("{}{}", proc_res.stdout, proc_res.stderr)
        } else {
            proc_res.stderr.clone()
        }
    }

    fn check_correct_failure_status(&self, proc_res: &ProcRes) {
        let expected_status = Some(self.props.failure_status.unwrap_or(1));
        let received_status = proc_res.status.code();

        if expected_status != received_status {
            self.fatal_proc_rec(
                &format!(
                    "Error: expected failure status ({:?}) but received status {:?}.",
                    expected_status, received_status
                ),
                proc_res,
            );
        }
    }

    fn run_cpass_test(&self) {
        let emit_metadata = self.should_emit_metadata(self.pass_mode());
        let proc_res = self.compile_test(WillExecute::No, emit_metadata);

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        // FIXME(#41968): Move this check to tidy?
        if !errors::load_errors(&self.testpaths.file, self.revision).is_empty() {
            self.fatal("compile-pass tests with expected warnings should be moved to ui/");
        }
    }

    fn run_rpass_test(&self) {
        let emit_metadata = self.should_emit_metadata(self.pass_mode());
        let should_run = self.run_if_enabled();
        let proc_res = self.compile_test(should_run, emit_metadata);

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        // FIXME(#41968): Move this check to tidy?
        if !errors::load_errors(&self.testpaths.file, self.revision).is_empty() {
            self.fatal("run-pass tests with expected warnings should be moved to ui/");
        }

        if let WillExecute::Disabled = should_run {
            return;
        }

        let proc_res = self.exec_compiled_test();
        if !proc_res.status.success() {
            self.fatal_proc_rec("test run failed!", &proc_res);
        }
    }

    fn run_valgrind_test(&self) {
        assert!(self.revision.is_none(), "revisions not relevant here");

        if self.config.valgrind_path.is_none() {
            assert!(!self.config.force_valgrind);
            return self.run_rpass_test();
        }

        let should_run = self.run_if_enabled();
        let mut proc_res = self.compile_test(should_run, Emit::None);

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        if let WillExecute::Disabled = should_run {
            return;
        }

        let mut new_config = self.config.clone();
        new_config.runtool = new_config.valgrind_path.clone();
        let new_cx = TestCx { config: &new_config, ..*self };
        proc_res = new_cx.exec_compiled_test();

        if !proc_res.status.success() {
            self.fatal_proc_rec("test run failed!", &proc_res);
        }
    }

    fn run_coverage_test(&self) {
        let should_run = self.run_if_enabled();
        let proc_res = self.compile_test(should_run, Emit::None);

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }
        drop(proc_res);

        if let WillExecute::Disabled = should_run {
            return;
        }

        let profraw_path = self.output_base_dir().join("default.profraw");
        let profdata_path = self.output_base_dir().join("default.profdata");

        // Delete any existing profraw/profdata files to rule out unintended
        // interference between repeated test runs.
        if profraw_path.exists() {
            std::fs::remove_file(&profraw_path).unwrap();
        }
        if profdata_path.exists() {
            std::fs::remove_file(&profdata_path).unwrap();
        }

        let proc_res = self.exec_compiled_test_general(
            &[("LLVM_PROFILE_FILE", &profraw_path.to_str().unwrap())],
            false,
        );
        if self.props.failure_status.is_some() {
            self.check_correct_failure_status(&proc_res);
        } else if !proc_res.status.success() {
            self.fatal_proc_rec("test run failed!", &proc_res);
        }
        drop(proc_res);

        let mut profraw_paths = vec![profraw_path];
        let mut bin_paths = vec![self.make_exe_name()];

        if self.config.suite == "run-coverage-rustdoc" {
            self.run_doctests_for_coverage(&mut profraw_paths, &mut bin_paths);
        }

        // Run `llvm-profdata merge` to index the raw coverage output.
        let proc_res = self.run_llvm_tool("llvm-profdata", |cmd| {
            cmd.args(["merge", "--sparse", "--output"]);
            cmd.arg(&profdata_path);
            cmd.args(&profraw_paths);
        });
        if !proc_res.status.success() {
            self.fatal_proc_rec("llvm-profdata merge failed!", &proc_res);
        }
        drop(proc_res);

        // Run `llvm-cov show` to produce a coverage report in text format.
        let proc_res = self.run_llvm_tool("llvm-cov", |cmd| {
            cmd.args(["show", "--format=text", "--show-line-counts-or-regions"]);

            cmd.arg("--Xdemangler");
            cmd.arg(self.config.rust_demangler_path.as_ref().unwrap());

            cmd.arg("--instr-profile");
            cmd.arg(&profdata_path);

            for bin in &bin_paths {
                cmd.arg("--object");
                cmd.arg(bin);
            }
        });
        if !proc_res.status.success() {
            self.fatal_proc_rec("llvm-cov show failed!", &proc_res);
        }

        let kind = UI_COVERAGE;

        let expected_coverage = self.load_expected_output(kind);
        let normalized_actual_coverage =
            self.normalize_coverage_output(&proc_res.stdout).unwrap_or_else(|err| {
                self.fatal_proc_rec(&err, &proc_res);
            });

        let coverage_errors = self.compare_output(
            kind,
            &normalized_actual_coverage,
            &expected_coverage,
            self.props.compare_output_lines_by_subset,
        );

        if coverage_errors > 0 {
            self.fatal_proc_rec(
                &format!("{} errors occurred comparing coverage output.", coverage_errors),
                &proc_res,
            );
        }
    }

    /// Run any doctests embedded in this test file, and add any resulting
    /// `.profraw` files and doctest executables to the given vectors.
    fn run_doctests_for_coverage(
        &self,
        profraw_paths: &mut Vec<PathBuf>,
        bin_paths: &mut Vec<PathBuf>,
    ) {
        // Put .profraw files and doctest executables in dedicated directories,
        // to make it easier to glob them all later.
        let profraws_dir = self.output_base_dir().join("doc_profraws");
        let bins_dir = self.output_base_dir().join("doc_bins");

        // Remove existing directories to prevent cross-run interference.
        if profraws_dir.try_exists().unwrap() {
            std::fs::remove_dir_all(&profraws_dir).unwrap();
        }
        if bins_dir.try_exists().unwrap() {
            std::fs::remove_dir_all(&bins_dir).unwrap();
        }

        let mut rustdoc_cmd =
            Command::new(self.config.rustdoc_path.as_ref().expect("--rustdoc-path not passed"));

        // In general there will be multiple doctest binaries running, so we
        // tell the profiler runtime to write their coverage data into separate
        // profraw files.
        rustdoc_cmd.env("LLVM_PROFILE_FILE", profraws_dir.join("%p-%m.profraw"));

        rustdoc_cmd.args(["--test", "-Cinstrument-coverage"]);

        // Without this, the doctests complain about not being able to find
        // their enclosing file's crate for some reason.
        rustdoc_cmd.args(["--crate-name", "workaround_for_79771"]);

        // Persist the doctest binaries so that `llvm-cov show` can read their
        // embedded coverage mappings later.
        rustdoc_cmd.arg("-Zunstable-options");
        rustdoc_cmd.arg("--persist-doctests");
        rustdoc_cmd.arg(&bins_dir);

        rustdoc_cmd.arg("-L");
        rustdoc_cmd.arg(self.aux_output_dir_name());

        rustdoc_cmd.arg(&self.testpaths.file);

        let proc_res = self.compose_and_run_compiler(rustdoc_cmd, None);
        if !proc_res.status.success() {
            self.fatal_proc_rec("rustdoc --test failed!", &proc_res)
        }

        fn glob_iter(path: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
            let path_str = path.as_ref().to_str().unwrap();
            let iter = glob(path_str).unwrap();
            iter.map(Result::unwrap)
        }

        // Find all profraw files in the profraw directory.
        for p in glob_iter(profraws_dir.join("*.profraw")) {
            profraw_paths.push(p);
        }
        // Find all executables in the `--persist-doctests` directory, while
        // avoiding other file types (e.g. `.pdb` on Windows). This doesn't
        // need to be perfect, as long as it can handle the files actually
        // produced by `rustdoc --test`.
        for p in glob_iter(bins_dir.join("**/*")) {
            let is_bin = p.is_file()
                && match p.extension() {
                    None => true,
                    Some(ext) => ext == OsStr::new("exe"),
                };
            if is_bin {
                bin_paths.push(p);
            }
        }
    }

    fn run_llvm_tool(&self, name: &str, configure_cmd_fn: impl FnOnce(&mut Command)) -> ProcRes {
        let tool_path = self
            .config
            .llvm_bin_dir
            .as_ref()
            .expect("this test expects the LLVM bin dir to be available")
            .join(name);

        let mut cmd = Command::new(tool_path);
        configure_cmd_fn(&mut cmd);

        let output = cmd.output().unwrap_or_else(|_| panic!("failed to exec `{cmd:?}`"));

        let proc_res = ProcRes {
            status: output.status,
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
            cmdline: format!("{cmd:?}"),
        };
        self.dump_output(&proc_res.stdout, &proc_res.stderr);

        proc_res
    }

    fn normalize_coverage_output(&self, coverage: &str) -> Result<String, String> {
        let normalized = self.normalize_output(coverage, &[]);

        let mut lines = normalized.lines().collect::<Vec<_>>();

        Self::sort_coverage_file_sections(&mut lines)?;
        Self::sort_coverage_subviews(&mut lines)?;

        let joined_lines = lines.iter().flat_map(|line| [line, "\n"]).collect::<String>();
        Ok(joined_lines)
    }

    /// Coverage reports can describe multiple source files, separated by
    /// blank lines. The order of these files is unpredictable (since it
    /// depends on implementation details), so we need to sort the file
    /// sections into a consistent order before comparing against a snapshot.
    fn sort_coverage_file_sections(coverage_lines: &mut Vec<&str>) -> Result<(), String> {
        // Group the lines into file sections, separated by blank lines.
        let mut sections = coverage_lines.split(|line| line.is_empty()).collect::<Vec<_>>();

        // The last section should be empty, representing an extra trailing blank line.
        if !sections.last().is_some_and(|last| last.is_empty()) {
            return Err("coverage report should end with an extra blank line".to_owned());
        }

        // Sort the file sections (not including the final empty "section").
        let except_last = sections.len() - 1;
        (&mut sections[..except_last]).sort();

        // Join the file sections back into a flat list of lines, with
        // sections separated by blank lines.
        let joined = sections.join(&[""] as &[_]);
        assert_eq!(joined.len(), coverage_lines.len());
        *coverage_lines = joined;

        Ok(())
    }

    fn sort_coverage_subviews(coverage_lines: &mut Vec<&str>) -> Result<(), String> {
        let mut output_lines = Vec::new();

        // We accumulate a list of zero or more "subviews", where each
        // subview is a list of one or more lines.
        let mut subviews: Vec<Vec<&str>> = Vec::new();

        fn flush<'a>(subviews: &mut Vec<Vec<&'a str>>, output_lines: &mut Vec<&'a str>) {
            if subviews.is_empty() {
                return;
            }

            // Take and clear the list of accumulated subviews.
            let mut subviews = std::mem::take(subviews);

            // The last "subview" should be just a boundary line on its own,
            // so exclude it when sorting the other subviews.
            let except_last = subviews.len() - 1;
            (&mut subviews[..except_last]).sort();

            for view in subviews {
                for line in view {
                    output_lines.push(line);
                }
            }
        }

        for (line, line_num) in coverage_lines.iter().zip(1..) {
            if line.starts_with("  ------------------") {
                // This is a subview boundary line, so start a new subview.
                subviews.push(vec![line]);
            } else if line.starts_with("  |") {
                // Add this line to the current subview.
                subviews
                    .last_mut()
                    .ok_or(format!(
                        "unexpected subview line outside of a subview on line {line_num}"
                    ))?
                    .push(line);
            } else {
                // This line is not part of a subview, so sort and print any
                // accumulated subviews, and then print the line as-is.
                flush(&mut subviews, &mut output_lines);
                output_lines.push(line);
            }
        }

        flush(&mut subviews, &mut output_lines);
        assert!(subviews.is_empty());

        assert_eq!(output_lines.len(), coverage_lines.len());
        *coverage_lines = output_lines;

        Ok(())
    }

    fn run_pretty_test(&self) {
        if self.props.pp_exact.is_some() {
            logv(self.config, "testing for exact pretty-printing".to_owned());
        } else {
            logv(self.config, "testing for converging pretty-printing".to_owned());
        }

        let rounds = match self.props.pp_exact {
            Some(_) => 1,
            None => 2,
        };

        let src = fs::read_to_string(&self.testpaths.file).unwrap();
        let mut srcs = vec![src];

        let mut round = 0;
        while round < rounds {
            logv(
                self.config,
                format!("pretty-printing round {} revision {:?}", round, self.revision),
            );
            let read_from =
                if round == 0 { ReadFrom::Path } else { ReadFrom::Stdin(srcs[round].to_owned()) };

            let proc_res = self.print_source(read_from, &self.props.pretty_mode);
            if !proc_res.status.success() {
                self.fatal_proc_rec(
                    &format!(
                        "pretty-printing failed in round {} revision {:?}",
                        round, self.revision
                    ),
                    &proc_res,
                );
            }

            let ProcRes { stdout, .. } = proc_res;
            srcs.push(stdout);
            round += 1;
        }

        let mut expected = match self.props.pp_exact {
            Some(ref file) => {
                let filepath = self.testpaths.file.parent().unwrap().join(file);
                fs::read_to_string(&filepath).unwrap()
            }
            None => srcs[srcs.len() - 2].clone(),
        };
        let mut actual = srcs[srcs.len() - 1].clone();

        if self.props.pp_exact.is_some() {
            // Now we have to care about line endings
            let cr = "\r".to_owned();
            actual = actual.replace(&cr, "");
            expected = expected.replace(&cr, "");
        }

        if !self.config.bless {
            self.compare_source(&expected, &actual);
        } else if expected != actual {
            let filepath_buf;
            let filepath = match &self.props.pp_exact {
                Some(file) => {
                    filepath_buf = self.testpaths.file.parent().unwrap().join(file);
                    &filepath_buf
                }
                None => &self.testpaths.file,
            };
            fs::write(filepath, &actual).unwrap();
        }

        // If we're only making sure that the output matches then just stop here
        if self.props.pretty_compare_only {
            return;
        }

        // Finally, let's make sure it actually appears to remain valid code
        let proc_res = self.typecheck_source(actual);
        if !proc_res.status.success() {
            self.fatal_proc_rec("pretty-printed source does not typecheck", &proc_res);
        }

        if !self.props.pretty_expanded {
            return;
        }

        // additionally, run `-Zunpretty=expanded` and try to build it.
        let proc_res = self.print_source(ReadFrom::Path, "expanded");
        if !proc_res.status.success() {
            self.fatal_proc_rec("pretty-printing (expanded) failed", &proc_res);
        }

        let ProcRes { stdout: expanded_src, .. } = proc_res;
        let proc_res = self.typecheck_source(expanded_src);
        if !proc_res.status.success() {
            self.fatal_proc_rec("pretty-printed source (expanded) does not typecheck", &proc_res);
        }
    }

    fn print_source(&self, read_from: ReadFrom, pretty_type: &str) -> ProcRes {
        let aux_dir = self.aux_output_dir_name();
        let input: &str = match read_from {
            ReadFrom::Stdin(_) => "-",
            ReadFrom::Path => self.testpaths.file.to_str().unwrap(),
        };

        let mut rustc = Command::new(&self.config.rustc_path);
        rustc
            .arg(input)
            .args(&["-Z", &format!("unpretty={}", pretty_type)])
            .args(&["--target", &self.config.target])
            .arg("-L")
            .arg(&aux_dir)
            .args(&self.props.compile_flags)
            .envs(self.props.rustc_env.clone());
        self.maybe_add_external_args(&mut rustc, &self.config.target_rustcflags);

        let src = match read_from {
            ReadFrom::Stdin(src) => Some(src),
            ReadFrom::Path => None,
        };

        self.compose_and_run(
            rustc,
            self.config.compile_lib_path.to_str().unwrap(),
            Some(aux_dir.to_str().unwrap()),
            src,
        )
    }

    fn compare_source(&self, expected: &str, actual: &str) {
        if expected != actual {
            self.fatal(&format!(
                "pretty-printed source does not match expected source\n\
                 expected:\n\
                 ------------------------------------------\n\
                 {}\n\
                 ------------------------------------------\n\
                 actual:\n\
                 ------------------------------------------\n\
                 {}\n\
                 ------------------------------------------\n\
                 diff:\n\
                 ------------------------------------------\n\
                 {}\n",
                expected,
                actual,
                write_diff(expected, actual, 3),
            ));
        }
    }

    fn set_revision_flags(&self, cmd: &mut Command) {
        if let Some(revision) = self.revision {
            // Normalize revisions to be lowercase and replace `-`s with `_`s.
            // Otherwise the `--cfg` flag is not valid.
            let normalized_revision = revision.to_lowercase().replace("-", "_");
            cmd.args(&["--cfg", &normalized_revision]);
        }
    }

    fn typecheck_source(&self, src: String) -> ProcRes {
        let mut rustc = Command::new(&self.config.rustc_path);

        let out_dir = self.output_base_name().with_extension("pretty-out");
        let _ = fs::remove_dir_all(&out_dir);
        create_dir_all(&out_dir).unwrap();

        let target = if self.props.force_host { &*self.config.host } else { &*self.config.target };

        let aux_dir = self.aux_output_dir_name();

        rustc
            .arg("-")
            .arg("-Zno-codegen")
            .arg("--out-dir")
            .arg(&out_dir)
            .arg(&format!("--target={}", target))
            .arg("-L")
            .arg(&self.config.build_base)
            .arg("-L")
            .arg(aux_dir);
        self.set_revision_flags(&mut rustc);
        self.maybe_add_external_args(&mut rustc, &self.config.target_rustcflags);
        rustc.args(&self.props.compile_flags);

        self.compose_and_run_compiler(rustc, Some(src))
    }

    fn run_debuginfo_test(&self) {
        match self.config.debugger.unwrap() {
            Debugger::Cdb => self.run_debuginfo_cdb_test(),
            Debugger::Gdb => self.run_debuginfo_gdb_test(),
            Debugger::Lldb => self.run_debuginfo_lldb_test(),
        }
    }

    fn run_debuginfo_cdb_test(&self) {
        let config = Config {
            target_rustcflags: self.cleanup_debug_info_options(&self.config.target_rustcflags),
            host_rustcflags: self.cleanup_debug_info_options(&self.config.host_rustcflags),
            ..self.config.clone()
        };

        let test_cx = TestCx { config: &config, ..*self };

        test_cx.run_debuginfo_cdb_test_no_opt();
    }

    fn run_debuginfo_cdb_test_no_opt(&self) {
        let exe_file = self.make_exe_name();

        // Existing PDB files are update in-place. When changing the debuginfo
        // the compiler generates for something, this can lead to the situation
        // where both the old and the new version of the debuginfo for the same
        // type is present in the PDB, which is very confusing.
        // Therefore we delete any existing PDB file before compiling the test
        // case.
        // FIXME: If can reliably detect that MSVC's link.exe is used, then
        //        passing `/INCREMENTAL:NO` might be a cleaner way to do this.
        let pdb_file = exe_file.with_extension(".pdb");
        if pdb_file.exists() {
            std::fs::remove_file(pdb_file).unwrap();
        }

        // compile test file (it should have 'compile-flags:-g' in the header)
        let should_run = self.run_if_enabled();
        let compile_result = self.compile_test(should_run, Emit::None);
        if !compile_result.status.success() {
            self.fatal_proc_rec("compilation failed!", &compile_result);
        }
        if let WillExecute::Disabled = should_run {
            return;
        }

        let prefixes = {
            static PREFIXES: &[&str] = &["cdb", "cdbg"];
            // No "native rust support" variation for CDB yet.
            PREFIXES
        };

        // Parse debugger commands etc from test files
        let dbg_cmds = DebuggerCommands::parse_from(
            &self.testpaths.file,
            self.config,
            prefixes,
            self.revision,
        )
        .unwrap_or_else(|e| self.fatal(&e));

        // https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/debugger-commands
        let mut script_str = String::with_capacity(2048);
        script_str.push_str("version\n"); // List CDB (and more) version info in test output
        script_str.push_str(".nvlist\n"); // List loaded `*.natvis` files, bulk of custom MSVC debug

        // If a .js file exists next to the source file being tested, then this is a JavaScript
        // debugging extension that needs to be loaded.
        let mut js_extension = self.testpaths.file.clone();
        js_extension.set_extension("cdb.js");
        if js_extension.exists() {
            script_str.push_str(&format!(".scriptload \"{}\"\n", js_extension.to_string_lossy()));
        }

        // Set breakpoints on every line that contains the string "#break"
        let source_file_name = self.testpaths.file.file_name().unwrap().to_string_lossy();
        for line in &dbg_cmds.breakpoint_lines {
            script_str.push_str(&format!("bp `{}:{}`\n", source_file_name, line));
        }

        // Append the other `cdb-command:`s
        for line in &dbg_cmds.commands {
            script_str.push_str(line);
            script_str.push_str("\n");
        }

        script_str.push_str("qq\n"); // Quit the debugger (including remote debugger, if any)

        // Write the script into a file
        debug!("script_str = {}", script_str);
        self.dump_output_file(&script_str, "debugger.script");
        let debugger_script = self.make_out_name("debugger.script");

        let cdb_path = &self.config.cdb.as_ref().unwrap();
        let mut cdb = Command::new(cdb_path);
        cdb.arg("-lines") // Enable source line debugging.
            .arg("-cf")
            .arg(&debugger_script)
            .arg(&exe_file);

        let debugger_run_result = self.compose_and_run(
            cdb,
            self.config.run_lib_path.to_str().unwrap(),
            None, // aux_path
            None, // input
        );

        if !debugger_run_result.status.success() {
            self.fatal_proc_rec("Error while running CDB", &debugger_run_result);
        }

        if let Err(e) = dbg_cmds.check_output(&debugger_run_result) {
            self.fatal_proc_rec(&e, &debugger_run_result);
        }
    }

    fn run_debuginfo_gdb_test(&self) {
        let config = Config {
            target_rustcflags: self.cleanup_debug_info_options(&self.config.target_rustcflags),
            host_rustcflags: self.cleanup_debug_info_options(&self.config.host_rustcflags),
            ..self.config.clone()
        };

        let test_cx = TestCx { config: &config, ..*self };

        test_cx.run_debuginfo_gdb_test_no_opt();
    }

    fn run_debuginfo_gdb_test_no_opt(&self) {
        let prefixes = if self.config.gdb_native_rust {
            // GDB with Rust
            static PREFIXES: &[&str] = &["gdb", "gdbr"];
            println!("NOTE: compiletest thinks it is using GDB with native rust support");
            PREFIXES
        } else {
            // Generic GDB
            static PREFIXES: &[&str] = &["gdb", "gdbg"];
            println!("NOTE: compiletest thinks it is using GDB without native rust support");
            PREFIXES
        };

        let dbg_cmds = DebuggerCommands::parse_from(
            &self.testpaths.file,
            self.config,
            prefixes,
            self.revision,
        )
        .unwrap_or_else(|e| self.fatal(&e));
        let mut cmds = dbg_cmds.commands.join("\n");

        // compile test file (it should have 'compile-flags:-g' in the header)
        let should_run = self.run_if_enabled();
        let compiler_run_result = self.compile_test(should_run, Emit::None);
        if !compiler_run_result.status.success() {
            self.fatal_proc_rec("compilation failed!", &compiler_run_result);
        }
        if let WillExecute::Disabled = should_run {
            return;
        }

        let exe_file = self.make_exe_name();

        let debugger_run_result;
        if is_android_gdb_target(&self.config.target) {
            cmds = cmds.replace("run", "continue");

            let tool_path = match self.config.android_cross_path.to_str() {
                Some(x) => x.to_owned(),
                None => self.fatal("cannot find android cross path"),
            };

            // write debugger script
            let mut script_str = String::with_capacity(2048);
            script_str.push_str(&format!("set charset {}\n", Self::charset()));
            script_str.push_str(&format!("set sysroot {}\n", tool_path));
            script_str.push_str(&format!("file {}\n", exe_file.to_str().unwrap()));
            script_str.push_str("target remote :5039\n");
            script_str.push_str(&format!(
                "set solib-search-path \
                 ./{}/stage2/lib/rustlib/{}/lib/\n",
                self.config.host, self.config.target
            ));
            for line in &dbg_cmds.breakpoint_lines {
                script_str.push_str(
                    format!(
                        "break {:?}:{}\n",
                        self.testpaths.file.file_name().unwrap().to_string_lossy(),
                        *line
                    )
                    .as_str(),
                );
            }
            script_str.push_str(&cmds);
            script_str.push_str("\nquit\n");

            debug!("script_str = {}", script_str);
            self.dump_output_file(&script_str, "debugger.script");

            let adb_path = &self.config.adb_path;

            Command::new(adb_path)
                .arg("push")
                .arg(&exe_file)
                .arg(&self.config.adb_test_dir)
                .status()
                .unwrap_or_else(|_| panic!("failed to exec `{:?}`", adb_path));

            Command::new(adb_path)
                .args(&["forward", "tcp:5039", "tcp:5039"])
                .status()
                .unwrap_or_else(|_| panic!("failed to exec `{:?}`", adb_path));

            let adb_arg = format!(
                "export LD_LIBRARY_PATH={}; \
                 gdbserver{} :5039 {}/{}",
                self.config.adb_test_dir.clone(),
                if self.config.target.contains("aarch64") { "64" } else { "" },
                self.config.adb_test_dir.clone(),
                exe_file.file_name().unwrap().to_str().unwrap()
            );

            debug!("adb arg: {}", adb_arg);
            let mut adb = Command::new(adb_path)
                .args(&["shell", &adb_arg])
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to exec `{:?}`", adb_path));

            // Wait for the gdbserver to print out "Listening on port ..."
            // at which point we know that it's started and then we can
            // execute the debugger below.
            let mut stdout = BufReader::new(adb.stdout.take().unwrap());
            let mut line = String::new();
            loop {
                line.truncate(0);
                stdout.read_line(&mut line).unwrap();
                if line.starts_with("Listening on port 5039") {
                    break;
                }
            }
            drop(stdout);

            let mut debugger_script = OsString::from("-command=");
            debugger_script.push(self.make_out_name("debugger.script"));
            let debugger_opts: &[&OsStr] =
                &["-quiet".as_ref(), "-batch".as_ref(), "-nx".as_ref(), &debugger_script];

            let gdb_path = self.config.gdb.as_ref().unwrap();
            let Output { status, stdout, stderr } = Command::new(&gdb_path)
                .args(debugger_opts)
                .output()
                .unwrap_or_else(|_| panic!("failed to exec `{:?}`", gdb_path));
            let cmdline = {
                let mut gdb = Command::new(&format!("{}-gdb", self.config.target));
                gdb.args(debugger_opts);
                let cmdline = self.make_cmdline(&gdb, "");
                logv(self.config, format!("executing {}", cmdline));
                cmdline
            };

            debugger_run_result = ProcRes {
                status,
                stdout: String::from_utf8(stdout).unwrap(),
                stderr: String::from_utf8(stderr).unwrap(),
                cmdline,
            };
            if adb.kill().is_err() {
                println!("Adb process is already finished.");
            }
        } else {
            let rust_src_root =
                self.config.find_rust_src_root().expect("Could not find Rust source root");
            let rust_pp_module_rel_path = Path::new("./src/etc");
            let rust_pp_module_abs_path =
                rust_src_root.join(rust_pp_module_rel_path).to_str().unwrap().to_owned();
            // write debugger script
            let mut script_str = String::with_capacity(2048);
            script_str.push_str(&format!("set charset {}\n", Self::charset()));
            script_str.push_str("show version\n");

            match self.config.gdb_version {
                Some(version) => {
                    println!("NOTE: compiletest thinks it is using GDB version {}", version);

                    if version > extract_gdb_version("7.4").unwrap() {
                        // Add the directory containing the pretty printers to
                        // GDB's script auto loading safe path
                        script_str.push_str(&format!(
                            "add-auto-load-safe-path {}\n",
                            rust_pp_module_abs_path.replace(r"\", r"\\")
                        ));

                        let output_base_dir = self.output_base_dir().to_str().unwrap().to_owned();

                        // Add the directory containing the output binary to
                        // include embedded pretty printers to GDB's script
                        // auto loading safe path
                        script_str.push_str(&format!(
                            "add-auto-load-safe-path {}\n",
                            output_base_dir.replace(r"\", r"\\")
                        ));
                    }
                }
                _ => {
                    println!(
                        "NOTE: compiletest does not know which version of \
                         GDB it is using"
                    );
                }
            }

            // The following line actually doesn't have to do anything with
            // pretty printing, it just tells GDB to print values on one line:
            script_str.push_str("set print pretty off\n");

            // Add the pretty printer directory to GDB's source-file search path
            script_str
                .push_str(&format!("directory {}\n", rust_pp_module_abs_path.replace(r"\", r"\\")));

            // Load the target executable
            script_str
                .push_str(&format!("file {}\n", exe_file.to_str().unwrap().replace(r"\", r"\\")));

            // Force GDB to print values in the Rust format.
            if self.config.gdb_native_rust {
                script_str.push_str("set language rust\n");
            }

            // Add line breakpoints
            for line in &dbg_cmds.breakpoint_lines {
                script_str.push_str(&format!(
                    "break '{}':{}\n",
                    self.testpaths.file.file_name().unwrap().to_string_lossy(),
                    *line
                ));
            }

            script_str.push_str(&cmds);
            script_str.push_str("\nquit\n");

            debug!("script_str = {}", script_str);
            self.dump_output_file(&script_str, "debugger.script");

            let mut debugger_script = OsString::from("-command=");
            debugger_script.push(self.make_out_name("debugger.script"));

            let debugger_opts: &[&OsStr] =
                &["-quiet".as_ref(), "-batch".as_ref(), "-nx".as_ref(), &debugger_script];

            let mut gdb = Command::new(self.config.gdb.as_ref().unwrap());
            let pythonpath = if let Ok(pp) = std::env::var("PYTHONPATH") {
                format!("{pp}:{rust_pp_module_abs_path}")
            } else {
                rust_pp_module_abs_path
            };
            gdb.args(debugger_opts).env("PYTHONPATH", pythonpath);

            debugger_run_result =
                self.compose_and_run(gdb, self.config.run_lib_path.to_str().unwrap(), None, None);
        }

        if !debugger_run_result.status.success() {
            self.fatal_proc_rec("gdb failed to execute", &debugger_run_result);
        }

        if let Err(e) = dbg_cmds.check_output(&debugger_run_result) {
            self.fatal_proc_rec(&e, &debugger_run_result);
        }
    }

    fn run_debuginfo_lldb_test(&self) {
        if self.config.lldb_python_dir.is_none() {
            self.fatal("Can't run LLDB test because LLDB's python path is not set.");
        }

        let config = Config {
            target_rustcflags: self.cleanup_debug_info_options(&self.config.target_rustcflags),
            host_rustcflags: self.cleanup_debug_info_options(&self.config.host_rustcflags),
            ..self.config.clone()
        };

        let test_cx = TestCx { config: &config, ..*self };

        test_cx.run_debuginfo_lldb_test_no_opt();
    }

    fn run_debuginfo_lldb_test_no_opt(&self) {
        // compile test file (it should have 'compile-flags:-g' in the header)
        let should_run = self.run_if_enabled();
        let compile_result = self.compile_test(should_run, Emit::None);
        if !compile_result.status.success() {
            self.fatal_proc_rec("compilation failed!", &compile_result);
        }
        if let WillExecute::Disabled = should_run {
            return;
        }

        let exe_file = self.make_exe_name();

        match self.config.lldb_version {
            Some(ref version) => {
                println!("NOTE: compiletest thinks it is using LLDB version {}", version);
            }
            _ => {
                println!(
                    "NOTE: compiletest does not know which version of \
                     LLDB it is using"
                );
            }
        }

        let prefixes = if self.config.lldb_native_rust {
            static PREFIXES: &[&str] = &["lldb", "lldbr"];
            println!("NOTE: compiletest thinks it is using LLDB with native rust support");
            PREFIXES
        } else {
            static PREFIXES: &[&str] = &["lldb", "lldbg"];
            println!("NOTE: compiletest thinks it is using LLDB without native rust support");
            PREFIXES
        };

        // Parse debugger commands etc from test files
        let dbg_cmds = DebuggerCommands::parse_from(
            &self.testpaths.file,
            self.config,
            prefixes,
            self.revision,
        )
        .unwrap_or_else(|e| self.fatal(&e));

        // Write debugger script:
        // We don't want to hang when calling `quit` while the process is still running
        let mut script_str = String::from("settings set auto-confirm true\n");

        // Make LLDB emit its version, so we have it documented in the test output
        script_str.push_str("version\n");

        // Switch LLDB into "Rust mode"
        let rust_src_root =
            self.config.find_rust_src_root().expect("Could not find Rust source root");
        let rust_pp_module_rel_path = Path::new("./src/etc/lldb_lookup.py");
        let rust_pp_module_abs_path =
            rust_src_root.join(rust_pp_module_rel_path).to_str().unwrap().to_owned();

        let rust_type_regexes = vec![
            "^(alloc::([a-z_]+::)+)String$",
            "^&(mut )?str$",
            "^&(mut )?\\[.+\\]$",
            "^(std::ffi::([a-z_]+::)+)OsString$",
            "^(alloc::([a-z_]+::)+)Vec<.+>$",
            "^(alloc::([a-z_]+::)+)VecDeque<.+>$",
            "^(alloc::([a-z_]+::)+)BTreeSet<.+>$",
            "^(alloc::([a-z_]+::)+)BTreeMap<.+>$",
            "^(std::collections::([a-z_]+::)+)HashMap<.+>$",
            "^(std::collections::([a-z_]+::)+)HashSet<.+>$",
            "^(alloc::([a-z_]+::)+)Rc<.+>$",
            "^(alloc::([a-z_]+::)+)Arc<.+>$",
            "^(core::([a-z_]+::)+)Cell<.+>$",
            "^(core::([a-z_]+::)+)Ref<.+>$",
            "^(core::([a-z_]+::)+)RefMut<.+>$",
            "^(core::([a-z_]+::)+)RefCell<.+>$",
            "^core::num::([a-z_]+::)*NonZero.+$",
        ];

        script_str
            .push_str(&format!("command script import {}\n", &rust_pp_module_abs_path[..])[..]);
        script_str.push_str("type synthetic add -l lldb_lookup.synthetic_lookup -x '.*' ");
        script_str.push_str("--category Rust\n");
        for type_regex in rust_type_regexes {
            script_str.push_str("type summary add -F lldb_lookup.summary_lookup  -e -x -h ");
            script_str.push_str(&format!("'{}' ", type_regex));
            script_str.push_str("--category Rust\n");
        }
        script_str.push_str("type category enable Rust\n");

        // Set breakpoints on every line that contains the string "#break"
        let source_file_name = self.testpaths.file.file_name().unwrap().to_string_lossy();
        for line in &dbg_cmds.breakpoint_lines {
            script_str.push_str(&format!(
                "breakpoint set --file '{}' --line {}\n",
                source_file_name, line
            ));
        }

        // Append the other commands
        for line in &dbg_cmds.commands {
            script_str.push_str(line);
            script_str.push_str("\n");
        }

        // Finally, quit the debugger
        script_str.push_str("\nquit\n");

        // Write the script into a file
        debug!("script_str = {}", script_str);
        self.dump_output_file(&script_str, "debugger.script");
        let debugger_script = self.make_out_name("debugger.script");

        // Let LLDB execute the script via lldb_batchmode.py
        let debugger_run_result = self.run_lldb(&exe_file, &debugger_script, &rust_src_root);

        if !debugger_run_result.status.success() {
            self.fatal_proc_rec("Error while running LLDB", &debugger_run_result);
        }

        if let Err(e) = dbg_cmds.check_output(&debugger_run_result) {
            self.fatal_proc_rec(&e, &debugger_run_result);
        }
    }

    fn run_lldb(
        &self,
        test_executable: &Path,
        debugger_script: &Path,
        rust_src_root: &Path,
    ) -> ProcRes {
        // Prepare the lldb_batchmode which executes the debugger script
        let lldb_script_path = rust_src_root.join("src/etc/lldb_batchmode.py");
        let pythonpath = if let Ok(pp) = std::env::var("PYTHONPATH") {
            format!("{pp}:{}", self.config.lldb_python_dir.as_ref().unwrap())
        } else {
            self.config.lldb_python_dir.as_ref().unwrap().to_string()
        };
        self.cmd2procres(
            Command::new(&self.config.python)
                .arg(&lldb_script_path)
                .arg(test_executable)
                .arg(debugger_script)
                .env("PYTHONUNBUFFERED", "1") // Help debugging #78665
                .env("PYTHONPATH", pythonpath),
        )
    }

    fn cmd2procres(&self, cmd: &mut Command) -> ProcRes {
        let (status, out, err) = match cmd.output() {
            Ok(Output { status, stdout, stderr }) => {
                (status, String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap())
            }
            Err(e) => self.fatal(&format!(
                "Failed to setup Python process for \
                 LLDB script: {}",
                e
            )),
        };

        self.dump_output(&out, &err);
        ProcRes { status, stdout: out, stderr: err, cmdline: format!("{:?}", cmd) }
    }

    fn cleanup_debug_info_options(&self, options: &Vec<String>) -> Vec<String> {
        // Remove options that are either unwanted (-O) or may lead to duplicates due to RUSTFLAGS.
        let options_to_remove = ["-O".to_owned(), "-g".to_owned(), "--debuginfo".to_owned()];

        options.iter().filter(|x| !options_to_remove.contains(x)).map(|x| x.clone()).collect()
    }

    fn maybe_add_external_args(&self, cmd: &mut Command, args: &Vec<String>) {
        // Filter out the arguments that should not be added by runtest here.
        //
        // Notable use-cases are: do not add our optimisation flag if
        // `compile-flags: -Copt-level=x` and similar for debug-info level as well.
        const OPT_FLAGS: &[&str] = &["-O", "-Copt-level=", /*-C<space>*/ "opt-level="];
        const DEBUG_FLAGS: &[&str] = &["-g", "-Cdebuginfo=", /*-C<space>*/ "debuginfo="];

        // FIXME: ideally we would "just" check the `cmd` itself, but it does not allow inspecting
        // its arguments. They need to be collected separately. For now I cannot be bothered to
        // implement this the "right" way.
        let have_opt_flag =
            self.props.compile_flags.iter().any(|arg| OPT_FLAGS.iter().any(|f| arg.starts_with(f)));
        let have_debug_flag = self
            .props
            .compile_flags
            .iter()
            .any(|arg| DEBUG_FLAGS.iter().any(|f| arg.starts_with(f)));

        for arg in args {
            if OPT_FLAGS.iter().any(|f| arg.starts_with(f)) && have_opt_flag {
                continue;
            }
            if DEBUG_FLAGS.iter().any(|f| arg.starts_with(f)) && have_debug_flag {
                continue;
            }
            cmd.arg(arg);
        }
    }

    fn check_all_error_patterns(
        &self,
        output_to_check: &str,
        proc_res: &ProcRes,
        pm: Option<PassMode>,
    ) {
        if self.props.error_patterns.is_empty() && self.props.regex_error_patterns.is_empty() {
            if pm.is_some() {
                // FIXME(#65865)
                return;
            } else {
                self.fatal(&format!(
                    "no error pattern specified in {:?}",
                    self.testpaths.file.display()
                ));
            }
        }

        let mut missing_patterns: Vec<String> = Vec::new();

        self.check_error_patterns(output_to_check, &mut missing_patterns);
        self.check_regex_error_patterns(output_to_check, proc_res, &mut missing_patterns);

        if missing_patterns.is_empty() {
            return;
        }

        if missing_patterns.len() == 1 {
            self.fatal_proc_rec(
                &format!("error pattern '{}' not found!", missing_patterns[0]),
                proc_res,
            );
        } else {
            for pattern in missing_patterns {
                self.error(&format!("error pattern '{}' not found!", pattern));
            }
            self.fatal_proc_rec("multiple error patterns not found", proc_res);
        }
    }

    fn check_error_patterns(&self, output_to_check: &str, missing_patterns: &mut Vec<String>) {
        debug!("check_error_patterns");
        for pattern in &self.props.error_patterns {
            if output_to_check.contains(pattern.trim()) {
                debug!("found error pattern {}", pattern);
            } else {
                missing_patterns.push(pattern.to_string());
            }
        }
    }

    fn check_regex_error_patterns(
        &self,
        output_to_check: &str,
        proc_res: &ProcRes,
        missing_patterns: &mut Vec<String>,
    ) {
        debug!("check_regex_error_patterns");

        for pattern in &self.props.regex_error_patterns {
            let pattern = pattern.trim();
            let re = match Regex::new(pattern) {
                Ok(re) => re,
                Err(err) => {
                    self.fatal_proc_rec(
                        &format!("invalid regex error pattern '{}': {:?}", pattern, err),
                        proc_res,
                    );
                }
            };
            if re.is_match(output_to_check) {
                debug!("found regex error pattern {}", pattern);
            } else {
                missing_patterns.push(pattern.to_string());
            }
        }
    }

    fn check_no_compiler_crash(&self, proc_res: &ProcRes, should_ice: bool) {
        match proc_res.status.code() {
            Some(101) if !should_ice => {
                self.fatal_proc_rec("compiler encountered internal error", proc_res)
            }
            None => self.fatal_proc_rec("compiler terminated by signal", proc_res),
            _ => (),
        }
    }

    fn check_forbid_output(&self, output_to_check: &str, proc_res: &ProcRes) {
        for pat in &self.props.forbid_output {
            if output_to_check.contains(pat) {
                self.fatal_proc_rec("forbidden pattern found in compiler output", proc_res);
            }
        }
    }

    fn check_expected_errors(&self, expected_errors: Vec<errors::Error>, proc_res: &ProcRes) {
        debug!(
            "check_expected_errors: expected_errors={:?} proc_res.status={:?}",
            expected_errors, proc_res.status
        );
        if proc_res.status.success()
            && expected_errors.iter().any(|x| x.kind == Some(ErrorKind::Error))
        {
            self.fatal_proc_rec("process did not return an error status", proc_res);
        }

        if self.props.known_bug {
            if !expected_errors.is_empty() {
                self.fatal_proc_rec(
                    "`known_bug` tests should not have an expected errors",
                    proc_res,
                );
            }
            return;
        }

        // On Windows, translate all '\' path separators to '/'
        let file_name = format!("{}", self.testpaths.file.display()).replace(r"\", "/");

        // On Windows, keep all '\' path separators to match the paths reported in the JSON output
        // from the compiler
        let diagnostic_file_name = if self.props.remap_src_base {
            let mut p = PathBuf::from(FAKE_SRC_BASE);
            p.push(&self.testpaths.relative_dir);
            p.push(self.testpaths.file.file_name().unwrap());
            p.display().to_string()
        } else {
            self.testpaths.file.display().to_string()
        };

        // If the testcase being checked contains at least one expected "help"
        // message, then we'll ensure that all "help" messages are expected.
        // Otherwise, all "help" messages reported by the compiler will be ignored.
        // This logic also applies to "note" messages.
        let expect_help = expected_errors.iter().any(|ee| ee.kind == Some(ErrorKind::Help));
        let expect_note = expected_errors.iter().any(|ee| ee.kind == Some(ErrorKind::Note));

        // Parse the JSON output from the compiler and extract out the messages.
        let actual_errors = json::parse_output(&diagnostic_file_name, &proc_res.stderr, proc_res);
        let mut unexpected = Vec::new();
        let mut found = vec![false; expected_errors.len()];
        for mut actual_error in actual_errors {
            actual_error.msg = self.normalize_output(&actual_error.msg, &[]);

            let opt_index =
                expected_errors.iter().enumerate().position(|(index, expected_error)| {
                    !found[index]
                        && actual_error.line_num == expected_error.line_num
                        && (expected_error.kind.is_none()
                            || actual_error.kind == expected_error.kind)
                        && actual_error.msg.contains(&expected_error.msg)
                });

            match opt_index {
                Some(index) => {
                    // found a match, everybody is happy
                    assert!(!found[index]);
                    found[index] = true;
                }

                None => {
                    // If the test is a known bug, don't require that the error is annotated
                    if self.is_unexpected_compiler_message(&actual_error, expect_help, expect_note)
                    {
                        self.error(&format!(
                            "{}:{}: unexpected {}: '{}'",
                            file_name,
                            actual_error.line_num,
                            actual_error
                                .kind
                                .as_ref()
                                .map_or(String::from("message"), |k| k.to_string()),
                            actual_error.msg
                        ));
                        unexpected.push(actual_error);
                    }
                }
            }
        }

        let mut not_found = Vec::new();
        // anything not yet found is a problem
        for (index, expected_error) in expected_errors.iter().enumerate() {
            if !found[index] {
                self.error(&format!(
                    "{}:{}: expected {} not found: {}",
                    file_name,
                    expected_error.line_num,
                    expected_error.kind.as_ref().map_or("message".into(), |k| k.to_string()),
                    expected_error.msg
                ));
                not_found.push(expected_error);
            }
        }

        if !unexpected.is_empty() || !not_found.is_empty() {
            self.error(&format!(
                "{} unexpected errors found, {} expected errors not found",
                unexpected.len(),
                not_found.len()
            ));
            println!("status: {}\ncommand: {}", proc_res.status, proc_res.cmdline);
            if !unexpected.is_empty() {
                println!("unexpected errors (from JSON output): {:#?}\n", unexpected);
            }
            if !not_found.is_empty() {
                println!("not found errors (from test file): {:#?}\n", not_found);
            }
            panic!();
        }
    }

    /// Returns `true` if we should report an error about `actual_error`,
    /// which did not match any of the expected error. We always require
    /// errors/warnings to be explicitly listed, but only require
    /// helps/notes if there are explicit helps/notes given.
    fn is_unexpected_compiler_message(
        &self,
        actual_error: &Error,
        expect_help: bool,
        expect_note: bool,
    ) -> bool {
        !actual_error.msg.is_empty()
            && match actual_error.kind {
                Some(ErrorKind::Help) => expect_help,
                Some(ErrorKind::Note) => expect_note,
                Some(ErrorKind::Error) | Some(ErrorKind::Warning) => true,
                Some(ErrorKind::Suggestion) | None => false,
            }
    }

    fn should_emit_metadata(&self, pm: Option<PassMode>) -> Emit {
        match (pm, self.props.fail_mode, self.config.mode) {
            (Some(PassMode::Check), ..) | (_, Some(FailMode::Check), Ui) => Emit::Metadata,
            _ => Emit::None,
        }
    }

    fn compile_test(&self, will_execute: WillExecute, emit: Emit) -> ProcRes {
        self.compile_test_general(will_execute, emit, self.props.local_pass_mode(), Vec::new())
    }

    fn compile_test_with_passes(
        &self,
        will_execute: WillExecute,
        emit: Emit,
        passes: Vec<String>,
    ) -> ProcRes {
        self.compile_test_general(will_execute, emit, self.props.local_pass_mode(), passes)
    }

    fn compile_test_general(
        &self,
        will_execute: WillExecute,
        emit: Emit,
        local_pm: Option<PassMode>,
        passes: Vec<String>,
    ) -> ProcRes {
        // Only use `make_exe_name` when the test ends up being executed.
        let output_file = match will_execute {
            WillExecute::Yes => TargetLocation::ThisFile(self.make_exe_name()),
            WillExecute::No | WillExecute::Disabled => {
                TargetLocation::ThisDirectory(self.output_base_dir())
            }
        };

        let allow_unused = match self.config.mode {
            Ui => {
                // UI tests tend to have tons of unused code as
                // it's just testing various pieces of the compile, but we don't
                // want to actually assert warnings about all this code. Instead
                // let's just ignore unused code warnings by defaults and tests
                // can turn it back on if needed.
                if !self.is_rustdoc()
                    // Note that we use the local pass mode here as we don't want
                    // to set unused to allow if we've overridden the pass mode
                    // via command line flags.
                    && local_pm != Some(PassMode::Run)
                {
                    AllowUnused::Yes
                } else {
                    AllowUnused::No
                }
            }
            _ => AllowUnused::No,
        };

        let rustc = self.make_compile_args(
            &self.testpaths.file,
            output_file,
            emit,
            allow_unused,
            LinkToAux::Yes,
            passes,
        );

        self.compose_and_run_compiler(rustc, None)
    }

    fn document(&self, out_dir: &Path) -> ProcRes {
        if self.props.build_aux_docs {
            for rel_ab in &self.props.aux_builds {
                let aux_testpaths = self.compute_aux_test_paths(rel_ab);
                let aux_props =
                    self.props.from_aux_file(&aux_testpaths.file, self.revision, self.config);
                let aux_cx = TestCx {
                    config: self.config,
                    props: &aux_props,
                    testpaths: &aux_testpaths,
                    revision: self.revision,
                };
                // Create the directory for the stdout/stderr files.
                create_dir_all(aux_cx.output_base_dir()).unwrap();
                let auxres = aux_cx.document(out_dir);
                if !auxres.status.success() {
                    return auxres;
                }
            }
        }

        let aux_dir = self.aux_output_dir_name();

        let rustdoc_path = self.config.rustdoc_path.as_ref().expect("--rustdoc-path not passed");
        let mut rustdoc = Command::new(rustdoc_path);

        rustdoc
            .arg("-L")
            .arg(self.config.run_lib_path.to_str().unwrap())
            .arg("-L")
            .arg(aux_dir)
            .arg("-o")
            .arg(out_dir)
            .arg("--deny")
            .arg("warnings")
            .arg(&self.testpaths.file)
            .args(&self.props.compile_flags);

        if self.config.mode == RustdocJson {
            rustdoc.arg("--output-format").arg("json").arg("-Zunstable-options");
        }

        if let Some(ref linker) = self.config.target_linker {
            rustdoc.arg(format!("-Clinker={}", linker));
        }

        self.compose_and_run_compiler(rustdoc, None)
    }

    fn exec_compiled_test(&self) -> ProcRes {
        self.exec_compiled_test_general(&[], true)
    }

    fn exec_compiled_test_general(
        &self,
        env_extra: &[(&str, &str)],
        delete_after_success: bool,
    ) -> ProcRes {
        let prepare_env = |cmd: &mut Command| {
            for key in &self.props.unset_exec_env {
                cmd.env_remove(key);
            }

            for (key, val) in &self.props.exec_env {
                cmd.env(key, val);
            }
            for (key, val) in env_extra {
                cmd.env(key, val);
            }
        };

        let proc_res = match &*self.config.target {
            // This is pretty similar to below, we're transforming:
            //
            //      program arg1 arg2
            //
            // into
            //
            //      remote-test-client run program 2 support-lib.so support-lib2.so arg1 arg2
            //
            // The test-client program will upload `program` to the emulator
            // along with all other support libraries listed (in this case
            // `support-lib.so` and `support-lib2.so`. It will then execute
            // the program on the emulator with the arguments specified
            // (in the environment we give the process) and then report back
            // the same result.
            _ if self.config.remote_test_client.is_some() => {
                let aux_dir = self.aux_output_dir_name();
                let ProcArgs { prog, args } = self.make_run_args();
                let mut support_libs = Vec::new();
                if let Ok(entries) = aux_dir.read_dir() {
                    for entry in entries {
                        let entry = entry.unwrap();
                        if !entry.path().is_file() {
                            continue;
                        }
                        support_libs.push(entry.path());
                    }
                }
                let mut test_client =
                    Command::new(self.config.remote_test_client.as_ref().unwrap());
                test_client
                    .args(&["run", &support_libs.len().to_string(), &prog])
                    .args(support_libs)
                    .args(args);

                prepare_env(&mut test_client);

                self.compose_and_run(
                    test_client,
                    self.config.run_lib_path.to_str().unwrap(),
                    Some(aux_dir.to_str().unwrap()),
                    None,
                )
            }
            _ if self.config.target.contains("vxworks") => {
                let aux_dir = self.aux_output_dir_name();
                let ProcArgs { prog, args } = self.make_run_args();
                let mut wr_run = Command::new("wr-run");
                wr_run.args(&[&prog]).args(args);

                prepare_env(&mut wr_run);

                self.compose_and_run(
                    wr_run,
                    self.config.run_lib_path.to_str().unwrap(),
                    Some(aux_dir.to_str().unwrap()),
                    None,
                )
            }
            _ => {
                let aux_dir = self.aux_output_dir_name();
                let ProcArgs { prog, args } = self.make_run_args();
                let mut program = Command::new(&prog);
                program.args(args).current_dir(&self.output_base_dir());

                prepare_env(&mut program);

                self.compose_and_run(
                    program,
                    self.config.run_lib_path.to_str().unwrap(),
                    Some(aux_dir.to_str().unwrap()),
                    None,
                )
            }
        };

        if delete_after_success && proc_res.status.success() {
            // delete the executable after running it to save space.
            // it is ok if the deletion failed.
            let _ = fs::remove_file(self.make_exe_name());
        }

        proc_res
    }

    /// For each `aux-build: foo/bar` annotation, we check to find the
    /// file in an `auxiliary` directory relative to the test itself.
    fn compute_aux_test_paths(&self, rel_ab: &str) -> TestPaths {
        let test_ab = self
            .testpaths
            .file
            .parent()
            .expect("test file path has no parent")
            .join("auxiliary")
            .join(rel_ab);
        if !test_ab.exists() {
            self.fatal(&format!("aux-build `{}` source not found", test_ab.display()))
        }

        TestPaths {
            file: test_ab,
            relative_dir: self
                .testpaths
                .relative_dir
                .join(self.output_testname_unique())
                .join("auxiliary")
                .join(rel_ab)
                .parent()
                .expect("aux-build path has no parent")
                .to_path_buf(),
        }
    }

    fn is_vxworks_pure_static(&self) -> bool {
        if self.config.target.contains("vxworks") {
            match env::var("RUST_VXWORKS_TEST_DYLINK") {
                Ok(s) => s != "1",
                _ => true,
            }
        } else {
            false
        }
    }

    fn is_vxworks_pure_dynamic(&self) -> bool {
        self.config.target.contains("vxworks") && !self.is_vxworks_pure_static()
    }

    fn build_all_auxiliary(&self, rustc: &mut Command) -> PathBuf {
        let aux_dir = self.aux_output_dir_name();

        if !self.props.aux_builds.is_empty() {
            let _ = fs::remove_dir_all(&aux_dir);
            create_dir_all(&aux_dir).unwrap();
        }

        for rel_ab in &self.props.aux_builds {
            self.build_auxiliary(rel_ab, &aux_dir);
        }

        for (aux_name, aux_path) in &self.props.aux_crates {
            let is_dylib = self.build_auxiliary(&aux_path, &aux_dir);
            let lib_name =
                get_lib_name(&aux_path.trim_end_matches(".rs").replace('-', "_"), is_dylib);
            rustc.arg("--extern").arg(format!("{}={}/{}", aux_name, aux_dir.display(), lib_name));
        }

        aux_dir
    }

    fn compose_and_run_compiler(&self, mut rustc: Command, input: Option<String>) -> ProcRes {
        let aux_dir = self.build_all_auxiliary(&mut rustc);
        self.props.unset_rustc_env.iter().fold(&mut rustc, Command::env_remove);
        rustc.envs(self.props.rustc_env.clone());
        self.compose_and_run(
            rustc,
            self.config.compile_lib_path.to_str().unwrap(),
            Some(aux_dir.to_str().unwrap()),
            input,
        )
    }

    /// Builds an aux dependency.
    ///
    /// Returns whether or not it is a dylib.
    fn build_auxiliary(&self, source_path: &str, aux_dir: &Path) -> bool {
        let aux_testpaths = self.compute_aux_test_paths(source_path);
        let aux_props = self.props.from_aux_file(&aux_testpaths.file, self.revision, self.config);
        let aux_output = TargetLocation::ThisDirectory(self.aux_output_dir_name());
        let aux_cx = TestCx {
            config: self.config,
            props: &aux_props,
            testpaths: &aux_testpaths,
            revision: self.revision,
        };
        // Create the directory for the stdout/stderr files.
        create_dir_all(aux_cx.output_base_dir()).unwrap();
        let input_file = &aux_testpaths.file;
        let mut aux_rustc = aux_cx.make_compile_args(
            input_file,
            aux_output,
            Emit::None,
            AllowUnused::No,
            LinkToAux::No,
            Vec::new(),
        );

        for key in &aux_props.unset_rustc_env {
            aux_rustc.env_remove(key);
        }
        aux_rustc.envs(aux_props.rustc_env.clone());

        let (dylib, crate_type) = if aux_props.no_prefer_dynamic {
            (true, None)
        } else if self.config.target.contains("emscripten")
            || (self.config.target.contains("musl")
                && !aux_props.force_host
                && !self.config.host.contains("musl"))
            || self.config.target.contains("wasm32")
            || self.config.target.contains("nvptx")
            || self.is_vxworks_pure_static()
            || self.config.target.contains("bpf")
            || !self.config.target_cfg().dynamic_linking
            || self.config.mode == RunCoverage
        {
            // We primarily compile all auxiliary libraries as dynamic libraries
            // to avoid code size bloat and large binaries as much as possible
            // for the test suite (otherwise including libstd statically in all
            // executables takes up quite a bit of space).
            //
            // For targets like MUSL or Emscripten, however, there is no support for
            // dynamic libraries so we just go back to building a normal library. Note,
            // however, that for MUSL if the library is built with `force_host` then
            // it's ok to be a dylib as the host should always support dylibs.
            //
            // Coverage tests want static linking by default so that coverage
            // mappings in auxiliary libraries can be merged into the final
            // executable.
            (false, Some("lib"))
        } else {
            (true, Some("dylib"))
        };

        if let Some(crate_type) = crate_type {
            aux_rustc.args(&["--crate-type", crate_type]);
        }

        aux_rustc.arg("-L").arg(&aux_dir);

        let auxres = aux_cx.compose_and_run(
            aux_rustc,
            aux_cx.config.compile_lib_path.to_str().unwrap(),
            Some(aux_dir.to_str().unwrap()),
            None,
        );
        if !auxres.status.success() {
            self.fatal_proc_rec(
                &format!(
                    "auxiliary build of {:?} failed to compile: ",
                    aux_testpaths.file.display()
                ),
                &auxres,
            );
        }
        dylib
    }

    fn read2_abbreviated(&self, child: Child) -> Output {
        let mut filter_paths_from_len = Vec::new();
        let mut add_path = |path: &Path| {
            let path = path.display().to_string();
            let windows = path.replace("\\", "\\\\");
            if windows != path {
                filter_paths_from_len.push(windows);
            }
            filter_paths_from_len.push(path);
        };

        // List of paths that will not be measured when determining whether the output is larger
        // than the output truncation threshold.
        //
        // Note: avoid adding a subdirectory of an already filtered directory here, otherwise the
        // same slice of text will be double counted and the truncation might not happen.
        add_path(&self.config.src_base);
        add_path(&self.config.build_base);

        read2_abbreviated(child, &filter_paths_from_len).expect("failed to read output")
    }

    fn compose_and_run(
        &self,
        mut command: Command,
        lib_path: &str,
        aux_path: Option<&str>,
        input: Option<String>,
    ) -> ProcRes {
        let cmdline = {
            let cmdline = self.make_cmdline(&command, lib_path);
            logv(self.config, format!("executing {}", cmdline));
            cmdline
        };

        command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());

        // Need to be sure to put both the lib_path and the aux path in the dylib
        // search path for the child.
        add_dylib_path(&mut command, iter::once(lib_path).chain(aux_path));

        let mut child = disable_error_reporting(|| command.spawn())
            .unwrap_or_else(|_| panic!("failed to exec `{:?}`", &command));
        if let Some(input) = input {
            child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
        }

        let Output { status, stdout, stderr } = self.read2_abbreviated(child);

        let result = ProcRes {
            status,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            cmdline,
        };

        self.dump_output(&result.stdout, &result.stderr);

        result
    }

    fn is_rustdoc(&self) -> bool {
        self.config.src_base.ends_with("rustdoc-ui")
            || self.config.src_base.ends_with("rustdoc-js")
            || self.config.src_base.ends_with("rustdoc-json")
    }

    fn make_compile_args(
        &self,
        input_file: &Path,
        output_file: TargetLocation,
        emit: Emit,
        allow_unused: AllowUnused,
        link_to_aux: LinkToAux,
        passes: Vec<String>, // Vec of passes under mir-opt test to be dumped
    ) -> Command {
        let is_aux = input_file.components().map(|c| c.as_os_str()).any(|c| c == "auxiliary");
        let is_rustdoc = self.is_rustdoc() && !is_aux;
        let mut rustc = if !is_rustdoc {
            Command::new(&self.config.rustc_path)
        } else {
            Command::new(&self.config.rustdoc_path.clone().expect("no rustdoc built yet"))
        };
        rustc.arg(input_file);

        // Use a single thread for efficiency and a deterministic error message order
        rustc.arg("-Zthreads=1");

        // Hide libstd sources from ui tests to make sure we generate the stderr
        // output that users will see.
        // Without this, we may be producing good diagnostics in-tree but users
        // will not see half the information.
        //
        // This also has the benefit of more effectively normalizing output between different
        // compilers, so that we don't have to know the `/rustc/$sha` output to normalize after the
        // fact.
        rustc.arg("-Zsimulate-remapped-rust-src-base=/rustc/FAKE_PREFIX");
        rustc.arg("-Ztranslate-remapped-path-to-local-path=no");

        // Optionally prevent default --sysroot if specified in test compile-flags.
        if !self.props.compile_flags.iter().any(|flag| flag.starts_with("--sysroot"))
            && !self.config.host_rustcflags.iter().any(|flag| flag == "--sysroot")
        {
            // In stage 0, make sure we use `stage0-sysroot` instead of the bootstrap sysroot.
            rustc.arg("--sysroot").arg(&self.config.sysroot_base);
        }

        // Optionally prevent default --target if specified in test compile-flags.
        let custom_target = self.props.compile_flags.iter().any(|x| x.starts_with("--target"));

        if !custom_target {
            let target =
                if self.props.force_host { &*self.config.host } else { &*self.config.target };

            rustc.arg(&format!("--target={}", target));
        }
        self.set_revision_flags(&mut rustc);

        if !is_rustdoc {
            if let Some(ref incremental_dir) = self.props.incremental_dir {
                rustc.args(&["-C", &format!("incremental={}", incremental_dir.display())]);
                rustc.args(&["-Z", "incremental-verify-ich"]);
            }

            if self.config.mode == CodegenUnits {
                rustc.args(&["-Z", "human_readable_cgu_names"]);
            }
        }

        if self.config.optimize_tests && !is_rustdoc {
            match self.config.mode {
                Ui => {
                    // If optimize-tests is true we still only want to optimize tests that actually get
                    // executed and that don't specify their own optimization levels.
                    // Note: aux libs don't have a pass-mode, so they won't get optimized
                    // unless compile-flags are set in the aux file.
                    if self.config.optimize_tests
                        && self.props.pass_mode(&self.config) == Some(PassMode::Run)
                        && !self
                            .props
                            .compile_flags
                            .iter()
                            .any(|arg| arg == "-O" || arg.contains("opt-level"))
                    {
                        rustc.arg("-O");
                    }
                }
                DebugInfo => { /* debuginfo tests must be unoptimized */ }
                RunCoverage => {
                    // Coverage reports are affected by optimization level, and
                    // the current snapshots assume no optimization by default.
                }
                _ => {
                    rustc.arg("-O");
                }
            }
        }

        match self.config.mode {
            Incremental => {
                // If we are extracting and matching errors in the new
                // fashion, then you want JSON mode. Old-skool error
                // patterns still match the raw compiler output.
                if self.props.error_patterns.is_empty()
                    && self.props.regex_error_patterns.is_empty()
                {
                    rustc.args(&["--error-format", "json"]);
                    rustc.args(&["--json", "future-incompat"]);
                }
                rustc.arg("-Zui-testing");
                rustc.arg("-Zdeduplicate-diagnostics=no");
            }
            Ui => {
                if !self.props.compile_flags.iter().any(|s| s.starts_with("--error-format")) {
                    rustc.args(&["--error-format", "json"]);
                    rustc.args(&["--json", "future-incompat"]);
                }
                rustc.arg("-Ccodegen-units=1");
                // Hide line numbers to reduce churn
                rustc.arg("-Zui-testing");
                rustc.arg("-Zdeduplicate-diagnostics=no");
                // #[cfg(not(bootstrap)] unconditionally pass flag after beta bump
                // since `ui-fulldeps --stage=1` builds using the stage 0 compiler,
                // which doesn't have this flag.
                if !(self.config.stage_id.starts_with("stage1-")
                    && self.config.suite == "ui-fulldeps")
                {
                    rustc.arg("-Zwrite-long-types-to-disk=no");
                }
                // FIXME: use this for other modes too, for perf?
                rustc.arg("-Cstrip=debuginfo");
            }
            MirOpt => {
                // We check passes under test to minimize the mir-opt test dump
                // if files_for_miropt_test parses the passes, we dump only those passes
                // otherwise we conservatively pass -Zdump-mir=all
                let zdump_arg = if !passes.is_empty() {
                    format!("-Zdump-mir={}", passes.join(" | "))
                } else {
                    "-Zdump-mir=all".to_string()
                };

                rustc.args(&[
                    "-Copt-level=1",
                    &zdump_arg,
                    "-Zvalidate-mir",
                    "-Zdump-mir-exclude-pass-number",
                ]);
                if let Some(pass) = &self.props.mir_unit_test {
                    rustc.args(&["-Zmir-opt-level=0", &format!("-Zmir-enable-passes=+{}", pass)]);
                } else {
                    rustc.args(&[
                        "-Zmir-opt-level=4",
                        "-Zmir-enable-passes=+ReorderBasicBlocks,+ReorderLocals",
                    ]);
                }

                let mir_dump_dir = self.get_mir_dump_dir();
                let _ = fs::remove_dir_all(&mir_dump_dir);
                create_dir_all(mir_dump_dir.as_path()).unwrap();
                let mut dir_opt = "-Zdump-mir-dir=".to_string();
                dir_opt.push_str(mir_dump_dir.to_str().unwrap());
                debug!("dir_opt: {:?}", dir_opt);

                rustc.arg(dir_opt);
            }
            RunCoverage => {
                rustc.arg("-Cinstrument-coverage");
            }
            RunPassValgrind | Pretty | DebugInfo | Codegen | Rustdoc | RustdocJson | RunMake
            | CodegenUnits | JsDocTest | Assembly => {
                // do not use JSON output
            }
        }

        if self.props.remap_src_base {
            rustc.arg(format!(
                "--remap-path-prefix={}={}",
                self.config.src_base.display(),
                FAKE_SRC_BASE,
            ));
        }

        match emit {
            Emit::None => {}
            Emit::Metadata if is_rustdoc => {}
            Emit::Metadata => {
                rustc.args(&["--emit", "metadata"]);
            }
            Emit::LlvmIr => {
                rustc.args(&["--emit", "llvm-ir"]);
            }
            Emit::Asm => {
                rustc.args(&["--emit", "asm"]);
            }
            Emit::LinkArgsAsm => {
                rustc.args(&["-Clink-args=--emit=asm"]);
            }
        }

        if !is_rustdoc {
            if self.config.target == "wasm32-unknown-unknown" || self.is_vxworks_pure_static() {
                // rustc.arg("-g"); // get any backtrace at all on errors
            } else if !self.props.no_prefer_dynamic {
                rustc.args(&["-C", "prefer-dynamic"]);
            }
        }

        match output_file {
            TargetLocation::ThisFile(path) => {
                rustc.arg("-o").arg(path);
            }
            TargetLocation::ThisDirectory(path) => {
                if is_rustdoc {
                    // `rustdoc` uses `-o` for the output directory.
                    rustc.arg("-o").arg(path);
                } else {
                    rustc.arg("--out-dir").arg(path);
                }
            }
        }

        match self.config.compare_mode {
            Some(CompareMode::Polonius) => {
                rustc.args(&["-Zpolonius"]);
            }
            Some(CompareMode::NextSolver) => {
                rustc.args(&["-Ztrait-solver=next"]);
            }
            Some(CompareMode::NextSolverCoherence) => {
                rustc.args(&["-Ztrait-solver=next-coherence"]);
            }
            Some(CompareMode::SplitDwarf) if self.config.target.contains("windows") => {
                rustc.args(&["-Csplit-debuginfo=unpacked", "-Zunstable-options"]);
            }
            Some(CompareMode::SplitDwarf) => {
                rustc.args(&["-Csplit-debuginfo=unpacked"]);
            }
            Some(CompareMode::SplitDwarfSingle) => {
                rustc.args(&["-Csplit-debuginfo=packed"]);
            }
            None => {}
        }

        // Add `-A unused` before `config` flags and in-test (`props`) flags, so that they can
        // overwrite this.
        if let AllowUnused::Yes = allow_unused {
            rustc.args(&["-A", "unused"]);
        }

        if self.props.force_host {
            self.maybe_add_external_args(&mut rustc, &self.config.host_rustcflags);
            if !is_rustdoc {
                if let Some(ref linker) = self.config.host_linker {
                    rustc.arg(format!("-Clinker={}", linker));
                }
            }
        } else {
            self.maybe_add_external_args(&mut rustc, &self.config.target_rustcflags);
            if !is_rustdoc {
                if let Some(ref linker) = self.config.target_linker {
                    rustc.arg(format!("-Clinker={}", linker));
                }
            }
        }

        // Use dynamic musl for tests because static doesn't allow creating dylibs
        if self.config.host.contains("musl") || self.is_vxworks_pure_dynamic() {
            rustc.arg("-Ctarget-feature=-crt-static");
        }

        if let LinkToAux::Yes = link_to_aux {
            rustc.arg("-L").arg(self.aux_output_dir_name());
        }

        rustc.args(&self.props.compile_flags);

        rustc
    }

    fn make_exe_name(&self) -> PathBuf {
        // Using a single letter here to keep the path length down for
        // Windows.  Some test names get very long.  rustc creates `rcgu`
        // files with the module name appended to it which can more than
        // double the length.
        let mut f = self.output_base_dir().join("a");
        // FIXME: This is using the host architecture exe suffix, not target!
        if self.config.target.contains("emscripten") {
            f = f.with_extra_extension("js");
        } else if self.config.target.contains("wasm32") {
            f = f.with_extra_extension("wasm");
        } else if self.config.target.contains("spirv") {
            f = f.with_extra_extension("spv");
        } else if !env::consts::EXE_SUFFIX.is_empty() {
            f = f.with_extra_extension(env::consts::EXE_SUFFIX);
        }
        f
    }

    fn make_run_args(&self) -> ProcArgs {
        // If we've got another tool to run under (valgrind),
        // then split apart its command
        let mut args = self.split_maybe_args(&self.config.runtool);

        // If this is emscripten, then run tests under nodejs
        if self.config.target.contains("emscripten") {
            if let Some(ref p) = self.config.nodejs {
                args.push(p.clone());
            } else {
                self.fatal("emscripten target requested and no NodeJS binary found (--nodejs)");
            }
        // If this is otherwise wasm, then run tests under nodejs with our
        // shim
        } else if self.config.target.contains("wasm32") {
            if let Some(ref p) = self.config.nodejs {
                args.push(p.clone());
            } else {
                self.fatal("wasm32 target requested and no NodeJS binary found (--nodejs)");
            }

            let src = self
                .config
                .src_base
                .parent()
                .unwrap() // chop off `ui`
                .parent()
                .unwrap(); // chop off `tests`
            args.push(src.join("src/etc/wasm32-shim.js").display().to_string());
        }

        let exe_file = self.make_exe_name();

        // FIXME (#9639): This needs to handle non-utf8 paths
        args.push(exe_file.to_str().unwrap().to_owned());

        // Add the arguments in the run_flags directive
        args.extend(self.split_maybe_args(&self.props.run_flags));

        let prog = args.remove(0);
        ProcArgs { prog, args }
    }

    fn split_maybe_args(&self, argstr: &Option<String>) -> Vec<String> {
        match *argstr {
            Some(ref s) => s
                .split(' ')
                .filter_map(|s| {
                    if s.chars().all(|c| c.is_whitespace()) { None } else { Some(s.to_owned()) }
                })
                .collect(),
            None => Vec::new(),
        }
    }

    fn make_cmdline(&self, command: &Command, libpath: &str) -> String {
        use crate::util;

        // Linux and mac don't require adjusting the library search path
        if cfg!(unix) {
            format!("{:?}", command)
        } else {
            // Build the LD_LIBRARY_PATH variable as it would be seen on the command line
            // for diagnostic purposes
            fn lib_path_cmd_prefix(path: &str) -> String {
                format!("{}=\"{}\"", util::lib_path_env_var(), util::make_new_path(path))
            }

            format!("{} {:?}", lib_path_cmd_prefix(libpath), command)
        }
    }

    fn dump_output(&self, out: &str, err: &str) {
        let revision = if let Some(r) = self.revision { format!("{}.", r) } else { String::new() };

        self.dump_output_file(out, &format!("{}out", revision));
        self.dump_output_file(err, &format!("{}err", revision));
        self.maybe_dump_to_stdout(out, err);
    }

    fn dump_output_file(&self, out: &str, extension: &str) {
        let outfile = self.make_out_name(extension);
        fs::write(&outfile, out).unwrap();
    }

    /// Creates a filename for output with the given extension.
    /// E.g., `/.../testname.revision.mode/testname.extension`.
    fn make_out_name(&self, extension: &str) -> PathBuf {
        self.output_base_name().with_extension(extension)
    }

    /// Gets the directory where auxiliary files are written.
    /// E.g., `/.../testname.revision.mode/auxiliary/`.
    fn aux_output_dir_name(&self) -> PathBuf {
        self.output_base_dir()
            .join("auxiliary")
            .with_extra_extension(self.config.mode.disambiguator())
    }

    /// Generates a unique name for the test, such as `testname.revision.mode`.
    fn output_testname_unique(&self) -> PathBuf {
        output_testname_unique(self.config, self.testpaths, self.safe_revision())
    }

    /// The revision, ignored for incremental compilation since it wants all revisions in
    /// the same directory.
    fn safe_revision(&self) -> Option<&str> {
        if self.config.mode == Incremental { None } else { self.revision }
    }

    /// Gets the absolute path to the directory where all output for the given
    /// test/revision should reside.
    /// E.g., `/path/to/build/host-triple/test/ui/relative/testname.revision.mode/`.
    fn output_base_dir(&self) -> PathBuf {
        output_base_dir(self.config, self.testpaths, self.safe_revision())
    }

    /// Gets the absolute path to the base filename used as output for the given
    /// test/revision.
    /// E.g., `/.../relative/testname.revision.mode/testname`.
    fn output_base_name(&self) -> PathBuf {
        output_base_name(self.config, self.testpaths, self.safe_revision())
    }

    fn maybe_dump_to_stdout(&self, out: &str, err: &str) {
        if self.config.verbose {
            println!("------stdout------------------------------");
            println!("{}", out);
            println!("------stderr------------------------------");
            println!("{}", err);
            println!("------------------------------------------");
        }
    }

    fn error(&self, err: &str) {
        match self.revision {
            Some(rev) => println!("\nerror in revision `{}`: {}", rev, err),
            None => println!("\nerror: {}", err),
        }
    }

    fn fatal(&self, err: &str) -> ! {
        self.error(err);
        error!("fatal error, panic: {:?}", err);
        panic!("fatal error");
    }

    fn fatal_proc_rec(&self, err: &str, proc_res: &ProcRes) -> ! {
        self.error(err);
        proc_res.fatal(None, || ());
    }

    fn fatal_proc_rec_with_ctx(
        &self,
        err: &str,
        proc_res: &ProcRes,
        on_failure: impl FnOnce(Self),
    ) -> ! {
        self.error(err);
        proc_res.fatal(None, || on_failure(*self));
    }

    // codegen tests (using FileCheck)

    fn compile_test_and_save_ir(&self) -> ProcRes {
        let output_file = TargetLocation::ThisDirectory(self.output_base_dir());
        let input_file = &self.testpaths.file;
        let rustc = self.make_compile_args(
            input_file,
            output_file,
            Emit::LlvmIr,
            AllowUnused::No,
            LinkToAux::Yes,
            Vec::new(),
        );

        self.compose_and_run_compiler(rustc, None)
    }

    fn compile_test_and_save_assembly(&self) -> (ProcRes, PathBuf) {
        // This works with both `--emit asm` (as default output name for the assembly)
        // and `ptx-linker` because the latter can write output at requested location.
        let output_path = self.output_base_name().with_extension("s");

        let output_file = TargetLocation::ThisFile(output_path.clone());
        let input_file = &self.testpaths.file;

        let mut emit = Emit::None;
        match self.props.assembly_output.as_ref().map(AsRef::as_ref) {
            Some("emit-asm") => {
                emit = Emit::Asm;
            }

            Some("bpf-linker") => {
                emit = Emit::LinkArgsAsm;
            }

            Some("ptx-linker") => {
                // No extra flags needed.
            }

            Some(header) => self.fatal(&format!("unknown 'assembly-output' header: {header}")),
            None => self.fatal("missing 'assembly-output' header"),
        }

        let rustc = self.make_compile_args(
            input_file,
            output_file,
            emit,
            AllowUnused::No,
            LinkToAux::Yes,
            Vec::new(),
        );

        (self.compose_and_run_compiler(rustc, None), output_path)
    }

    fn verify_with_filecheck(&self, output: &Path) -> ProcRes {
        let mut filecheck = Command::new(self.config.llvm_filecheck.as_ref().unwrap());
        filecheck.arg("--input-file").arg(output).arg(&self.testpaths.file);
        // It would be more appropriate to make most of the arguments configurable through
        // a comment-attribute similar to `compile-flags`. For example, --check-prefixes is a very
        // useful flag.
        //
        // For now, though…
        let prefix_for_target =
            if self.config.target.contains("msvc") { "MSVC" } else { "NONMSVC" };
        let prefixes = if let Some(rev) = self.revision {
            format!("CHECK,{},{}", prefix_for_target, rev)
        } else {
            format!("CHECK,{}", prefix_for_target)
        };
        if self.config.llvm_version.unwrap_or(0) >= 130000 {
            filecheck.args(&["--allow-unused-prefixes", "--check-prefixes", &prefixes]);
        } else {
            filecheck.args(&["--check-prefixes", &prefixes]);
        }
        // Provide more context on failures.
        filecheck.args(&["--dump-input-context", "100"]);
        self.compose_and_run(filecheck, "", None, None)
    }

    fn run_codegen_test(&self) {
        if self.config.llvm_filecheck.is_none() {
            self.fatal("missing --llvm-filecheck");
        }

        let proc_res = self.compile_test_and_save_ir();
        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        if let Some(PassMode::Build) = self.pass_mode() {
            return;
        }

        let output_path = self.output_base_name().with_extension("ll");
        let proc_res = self.verify_with_filecheck(&output_path);
        if !proc_res.status.success() {
            self.fatal_proc_rec("verification with 'FileCheck' failed", &proc_res);
        }
    }

    fn run_assembly_test(&self) {
        if self.config.llvm_filecheck.is_none() {
            self.fatal("missing --llvm-filecheck");
        }

        let (proc_res, output_path) = self.compile_test_and_save_assembly();
        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        let proc_res = self.verify_with_filecheck(&output_path);
        if !proc_res.status.success() {
            self.fatal_proc_rec("verification with 'FileCheck' failed", &proc_res);
        }
    }

    fn charset() -> &'static str {
        // FreeBSD 10.1 defaults to GDB 6.1.1 which doesn't support "auto" charset
        if cfg!(target_os = "freebsd") { "ISO-8859-1" } else { "UTF-8" }
    }

    fn run_rustdoc_test(&self) {
        assert!(self.revision.is_none(), "revisions not relevant here");

        let out_dir = self.output_base_dir();
        let _ = fs::remove_dir_all(&out_dir);
        create_dir_all(&out_dir).unwrap();

        let proc_res = self.document(&out_dir);
        if !proc_res.status.success() {
            self.fatal_proc_rec("rustdoc failed!", &proc_res);
        }

        if self.props.check_test_line_numbers_match {
            self.check_rustdoc_test_option(proc_res);
        } else {
            let root = self.config.find_rust_src_root().unwrap();
            let mut cmd = Command::new(&self.config.python);
            cmd.arg(root.join("src/etc/htmldocck.py")).arg(&out_dir).arg(&self.testpaths.file);
            if self.config.bless {
                cmd.arg("--bless");
            }
            let res = self.cmd2procres(&mut cmd);
            if !res.status.success() {
                self.fatal_proc_rec_with_ctx("htmldocck failed!", &res, |mut this| {
                    this.compare_to_default_rustdoc(&out_dir)
                });
            }
        }
    }

    fn compare_to_default_rustdoc(&mut self, out_dir: &Path) {
        if !self.config.has_tidy {
            return;
        }
        println!("info: generating a diff against nightly rustdoc");

        let suffix =
            self.safe_revision().map_or("nightly".into(), |path| path.to_owned() + "-nightly");
        let compare_dir = output_base_dir(self.config, self.testpaths, Some(&suffix));
        // Don't give an error if the directory didn't already exist
        let _ = fs::remove_dir_all(&compare_dir);
        create_dir_all(&compare_dir).unwrap();

        // We need to create a new struct for the lifetimes on `config` to work.
        let new_rustdoc = TestCx {
            config: &Config {
                // FIXME: use beta or a user-specified rustdoc instead of
                // hardcoding the default toolchain
                rustdoc_path: Some("rustdoc".into()),
                // Needed for building auxiliary docs below
                rustc_path: "rustc".into(),
                ..self.config.clone()
            },
            ..*self
        };

        let output_file = TargetLocation::ThisDirectory(new_rustdoc.aux_output_dir_name());
        let mut rustc = new_rustdoc.make_compile_args(
            &new_rustdoc.testpaths.file,
            output_file,
            Emit::None,
            AllowUnused::Yes,
            LinkToAux::Yes,
            Vec::new(),
        );
        new_rustdoc.build_all_auxiliary(&mut rustc);

        let proc_res = new_rustdoc.document(&compare_dir);
        if !proc_res.status.success() {
            eprintln!("failed to run nightly rustdoc");
            return;
        }

        #[rustfmt::skip]
        let tidy_args = [
            "--indent", "yes",
            "--indent-spaces", "2",
            "--wrap", "0",
            "--show-warnings", "no",
            "--markup", "yes",
            "--quiet", "yes",
            "-modify",
        ];
        let tidy_dir = |dir| {
            for entry in walkdir::WalkDir::new(dir) {
                let entry = entry.expect("failed to read file");
                if entry.file_type().is_file()
                    && entry.path().extension().and_then(|p| p.to_str()) == Some("html".into())
                {
                    let status =
                        Command::new("tidy").args(&tidy_args).arg(entry.path()).status().unwrap();
                    // `tidy` returns 1 if it modified the file.
                    assert!(status.success() || status.code() == Some(1));
                }
            }
        };
        tidy_dir(out_dir);
        tidy_dir(&compare_dir);

        let pager = {
            let output = Command::new("git").args(&["config", "--get", "core.pager"]).output().ok();
            output.and_then(|out| {
                if out.status.success() {
                    Some(String::from_utf8(out.stdout).expect("invalid UTF8 in git pager"))
                } else {
                    None
                }
            })
        };

        let diff_filename = format!("build/tmp/rustdoc-compare-{}.diff", std::process::id());

        if !write_filtered_diff(
            &diff_filename,
            out_dir,
            &compare_dir,
            self.config.verbose,
            |file_type, extension| {
                file_type.is_file()
                    && (extension == Some("html".into()) || extension == Some("js".into()))
            },
        ) {
            return;
        }

        match self.config.color {
            ColorConfig::AlwaysColor => colored::control::set_override(true),
            ColorConfig::NeverColor => colored::control::set_override(false),
            _ => {}
        }

        if let Some(pager) = pager {
            let pager = pager.trim();
            if self.config.verbose {
                eprintln!("using pager {}", pager);
            }
            let output = Command::new(pager)
                // disable paging; we want this to be non-interactive
                .env("PAGER", "")
                .stdin(File::open(&diff_filename).unwrap())
                // Capture output and print it explicitly so it will in turn be
                // captured by libtest.
                .output()
                .unwrap();
            assert!(output.status.success());
            println!("{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        } else {
            use colored::Colorize;
            eprintln!("warning: no pager configured, falling back to unified diff");
            eprintln!(
                "help: try configuring a git pager (e.g. `delta`) with `git config --global core.pager delta`"
            );
            let mut out = io::stdout();
            let mut diff = BufReader::new(File::open(&diff_filename).unwrap());
            let mut line = Vec::new();
            loop {
                line.truncate(0);
                match diff.read_until(b'\n', &mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) => eprintln!("ERROR: {:?}", e),
                }
                match String::from_utf8(line.clone()) {
                    Ok(line) => {
                        if line.starts_with("+") {
                            write!(&mut out, "{}", line.green()).unwrap();
                        } else if line.starts_with("-") {
                            write!(&mut out, "{}", line.red()).unwrap();
                        } else if line.starts_with("@") {
                            write!(&mut out, "{}", line.blue()).unwrap();
                        } else {
                            out.write_all(line.as_bytes()).unwrap();
                        }
                    }
                    Err(_) => {
                        write!(&mut out, "{}", String::from_utf8_lossy(&line).reversed()).unwrap();
                    }
                }
            }
        };
    }

    fn run_rustdoc_json_test(&self) {
        //FIXME: Add bless option.

        assert!(self.revision.is_none(), "revisions not relevant here");

        let out_dir = self.output_base_dir();
        let _ = fs::remove_dir_all(&out_dir);
        create_dir_all(&out_dir).unwrap();

        let proc_res = self.document(&out_dir);
        if !proc_res.status.success() {
            self.fatal_proc_rec("rustdoc failed!", &proc_res);
        }

        let root = self.config.find_rust_src_root().unwrap();
        let mut json_out = out_dir.join(self.testpaths.file.file_stem().unwrap());
        json_out.set_extension("json");
        let res = self.cmd2procres(
            Command::new(self.config.jsondocck_path.as_ref().unwrap())
                .arg("--doc-dir")
                .arg(root.join(&out_dir))
                .arg("--template")
                .arg(&self.testpaths.file),
        );

        if !res.status.success() {
            self.fatal_proc_rec_with_ctx("jsondocck failed!", &res, |_| {
                println!("Rustdoc Output:");
                proc_res.print_info();
            })
        }

        let mut json_out = out_dir.join(self.testpaths.file.file_stem().unwrap());
        json_out.set_extension("json");

        let res = self.cmd2procres(
            Command::new(self.config.jsondoclint_path.as_ref().unwrap()).arg(&json_out),
        );

        if !res.status.success() {
            self.fatal_proc_rec("jsondoclint failed!", &res);
        }
    }

    fn get_lines<P: AsRef<Path>>(
        &self,
        path: &P,
        mut other_files: Option<&mut Vec<String>>,
    ) -> Vec<usize> {
        let content = fs::read_to_string(&path).unwrap();
        let mut ignore = false;
        content
            .lines()
            .enumerate()
            .filter_map(|(line_nb, line)| {
                if (line.trim_start().starts_with("pub mod ")
                    || line.trim_start().starts_with("mod "))
                    && line.ends_with(';')
                {
                    if let Some(ref mut other_files) = other_files {
                        other_files.push(line.rsplit("mod ").next().unwrap().replace(";", ""));
                    }
                    None
                } else {
                    let sline = line.rsplit("///").next().unwrap();
                    let line = sline.trim_start();
                    if line.starts_with("```") {
                        if ignore {
                            ignore = false;
                            None
                        } else {
                            ignore = true;
                            Some(line_nb + 1)
                        }
                    } else {
                        None
                    }
                }
            })
            .collect()
    }

    fn check_rustdoc_test_option(&self, res: ProcRes) {
        let mut other_files = Vec::new();
        let mut files: HashMap<String, Vec<usize>> = HashMap::new();
        let cwd = env::current_dir().unwrap();
        files.insert(
            self.testpaths
                .file
                .strip_prefix(&cwd)
                .unwrap_or(&self.testpaths.file)
                .to_str()
                .unwrap()
                .replace('\\', "/"),
            self.get_lines(&self.testpaths.file, Some(&mut other_files)),
        );
        for other_file in other_files {
            let mut path = self.testpaths.file.clone();
            path.set_file_name(&format!("{}.rs", other_file));
            files.insert(
                path.strip_prefix(&cwd).unwrap_or(&path).to_str().unwrap().replace('\\', "/"),
                self.get_lines(&path, None),
            );
        }

        let mut tested = 0;
        for _ in res.stdout.split('\n').filter(|s| s.starts_with("test ")).inspect(|s| {
            if let Some((left, right)) = s.split_once(" - ") {
                let path = left.rsplit("test ").next().unwrap();
                if let Some(ref mut v) = files.get_mut(&path.replace('\\', "/")) {
                    tested += 1;
                    let mut iter = right.split("(line ");
                    iter.next();
                    let line = iter
                        .next()
                        .unwrap_or(")")
                        .split(')')
                        .next()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    if let Ok(pos) = v.binary_search(&line) {
                        v.remove(pos);
                    } else {
                        self.fatal_proc_rec(
                            &format!("Not found doc test: \"{}\" in \"{}\":{:?}", s, path, v),
                            &res,
                        );
                    }
                }
            }
        }) {}
        if tested == 0 {
            self.fatal_proc_rec(&format!("No test has been found... {:?}", files), &res);
        } else {
            for (entry, v) in &files {
                if !v.is_empty() {
                    self.fatal_proc_rec(
                        &format!(
                            "Not found test at line{} \"{}\":{:?}",
                            if v.len() > 1 { "s" } else { "" },
                            entry,
                            v
                        ),
                        &res,
                    );
                }
            }
        }
    }

    fn run_codegen_units_test(&self) {
        assert!(self.revision.is_none(), "revisions not relevant here");

        let proc_res = self.compile_test(WillExecute::No, Emit::None);

        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        self.check_no_compiler_crash(&proc_res, self.props.should_ice);

        const PREFIX: &str = "MONO_ITEM ";
        const CGU_MARKER: &str = "@@";

        let actual: Vec<MonoItem> = proc_res
            .stdout
            .lines()
            .filter(|line| line.starts_with(PREFIX))
            .map(|line| str_to_mono_item(line, true))
            .collect();

        let expected: Vec<MonoItem> = errors::load_errors(&self.testpaths.file, None)
            .iter()
            .map(|e| str_to_mono_item(&e.msg[..], false))
            .collect();

        let mut missing = Vec::new();
        let mut wrong_cgus = Vec::new();

        for expected_item in &expected {
            let actual_item_with_same_name = actual.iter().find(|ti| ti.name == expected_item.name);

            if let Some(actual_item) = actual_item_with_same_name {
                if !expected_item.codegen_units.is_empty() &&
                   // Also check for codegen units
                   expected_item.codegen_units != actual_item.codegen_units
                {
                    wrong_cgus.push((expected_item.clone(), actual_item.clone()));
                }
            } else {
                missing.push(expected_item.string.clone());
            }
        }

        let unexpected: Vec<_> = actual
            .iter()
            .filter(|acgu| !expected.iter().any(|ecgu| acgu.name == ecgu.name))
            .map(|acgu| acgu.string.clone())
            .collect();

        if !missing.is_empty() {
            missing.sort();

            println!("\nThese items should have been contained but were not:\n");

            for item in &missing {
                println!("{}", item);
            }

            println!("\n");
        }

        if !unexpected.is_empty() {
            let sorted = {
                let mut sorted = unexpected.clone();
                sorted.sort();
                sorted
            };

            println!("\nThese items were contained but should not have been:\n");

            for item in sorted {
                println!("{}", item);
            }

            println!("\n");
        }

        if !wrong_cgus.is_empty() {
            wrong_cgus.sort_by_key(|pair| pair.0.name.clone());
            println!("\nThe following items were assigned to wrong codegen units:\n");

            for &(ref expected_item, ref actual_item) in &wrong_cgus {
                println!("{}", expected_item.name);
                println!("  expected: {}", codegen_units_to_str(&expected_item.codegen_units));
                println!("  actual:   {}", codegen_units_to_str(&actual_item.codegen_units));
                println!();
            }
        }

        if !(missing.is_empty() && unexpected.is_empty() && wrong_cgus.is_empty()) {
            panic!();
        }

        #[derive(Clone, Eq, PartialEq)]
        struct MonoItem {
            name: String,
            codegen_units: HashSet<String>,
            string: String,
        }

        // [MONO_ITEM] name [@@ (cgu)+]
        fn str_to_mono_item(s: &str, cgu_has_crate_disambiguator: bool) -> MonoItem {
            let s = if s.starts_with(PREFIX) { (&s[PREFIX.len()..]).trim() } else { s.trim() };

            let full_string = format!("{}{}", PREFIX, s);

            let parts: Vec<&str> =
                s.split(CGU_MARKER).map(str::trim).filter(|s| !s.is_empty()).collect();

            let name = parts[0].trim();

            let cgus = if parts.len() > 1 {
                let cgus_str = parts[1];

                cgus_str
                    .split(' ')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        if cgu_has_crate_disambiguator {
                            remove_crate_disambiguators_from_set_of_cgu_names(s)
                        } else {
                            s.to_string()
                        }
                    })
                    .collect()
            } else {
                HashSet::new()
            };

            MonoItem { name: name.to_owned(), codegen_units: cgus, string: full_string }
        }

        fn codegen_units_to_str(cgus: &HashSet<String>) -> String {
            let mut cgus: Vec<_> = cgus.iter().collect();
            cgus.sort();

            let mut string = String::new();
            for cgu in cgus {
                string.push_str(&cgu[..]);
                string.push_str(" ");
            }

            string
        }

        // Given a cgu-name-prefix of the form <crate-name>.<crate-disambiguator> or
        // the form <crate-name1>.<crate-disambiguator1>-in-<crate-name2>.<crate-disambiguator2>,
        // remove all crate-disambiguators.
        fn remove_crate_disambiguator_from_cgu(cgu: &str) -> String {
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^[^\.]+(?P<d1>\.[[:alnum:]]+)(-in-[^\.]+(?P<d2>\.[[:alnum:]]+))?")
                    .unwrap()
            });

            let captures =
                RE.captures(cgu).unwrap_or_else(|| panic!("invalid cgu name encountered: {}", cgu));

            let mut new_name = cgu.to_owned();

            if let Some(d2) = captures.name("d2") {
                new_name.replace_range(d2.start()..d2.end(), "");
            }

            let d1 = captures.name("d1").unwrap();
            new_name.replace_range(d1.start()..d1.end(), "");

            new_name
        }

        // The name of merged CGUs is constructed as the names of the original
        // CGUs joined with "--". This function splits such composite CGU names
        // and handles each component individually.
        fn remove_crate_disambiguators_from_set_of_cgu_names(cgus: &str) -> String {
            cgus.split("--")
                .map(|cgu| remove_crate_disambiguator_from_cgu(cgu))
                .collect::<Vec<_>>()
                .join("--")
        }
    }

    fn init_incremental_test(&self) {
        // (See `run_incremental_test` for an overview of how incremental tests work.)

        // Before any of the revisions have executed, create the
        // incremental workproduct directory.  Delete any old
        // incremental work products that may be there from prior
        // runs.
        let incremental_dir = self.props.incremental_dir.as_ref().unwrap();
        if incremental_dir.exists() {
            // Canonicalizing the path will convert it to the //?/ format
            // on Windows, which enables paths longer than 260 character
            let canonicalized = incremental_dir.canonicalize().unwrap();
            fs::remove_dir_all(canonicalized).unwrap();
        }
        fs::create_dir_all(&incremental_dir).unwrap();

        if self.config.verbose {
            println!("init_incremental_test: incremental_dir={}", incremental_dir.display());
        }
    }

    fn run_incremental_test(&self) {
        // Basic plan for a test incremental/foo/bar.rs:
        // - load list of revisions rpass1, cfail2, rpass3
        //   - each should begin with `cpass`, `rpass`, `cfail`, or `rfail`
        //   - if `cpass`, expect compilation to succeed, don't execute
        //   - if `rpass`, expect compilation and execution to succeed
        //   - if `cfail`, expect compilation to fail
        //   - if `rfail`, expect compilation to succeed and execution to fail
        // - create a directory build/foo/bar.incremental
        // - compile foo/bar.rs with -C incremental=.../foo/bar.incremental and -C rpass1
        //   - because name of revision starts with "rpass", expect success
        // - compile foo/bar.rs with -C incremental=.../foo/bar.incremental and -C cfail2
        //   - because name of revision starts with "cfail", expect an error
        //   - load expected errors as usual, but filter for those that end in `[rfail2]`
        // - compile foo/bar.rs with -C incremental=.../foo/bar.incremental and -C rpass3
        //   - because name of revision starts with "rpass", expect success
        // - execute build/foo/bar.exe and save output
        //
        // FIXME -- use non-incremental mode as an oracle? That doesn't apply
        // to #[rustc_dirty] and clean tests I guess

        let revision = self.revision.expect("incremental tests require a list of revisions");

        // Incremental workproduct directory should have already been created.
        let incremental_dir = self.props.incremental_dir.as_ref().unwrap();
        assert!(incremental_dir.exists(), "init_incremental_test failed to create incremental dir");

        if self.config.verbose {
            print!("revision={:?} props={:#?}", revision, self.props);
        }

        if revision.starts_with("cpass") {
            if self.props.should_ice {
                self.fatal("can only use should-ice in cfail tests");
            }
            self.run_cpass_test();
        } else if revision.starts_with("rpass") {
            if self.props.should_ice {
                self.fatal("can only use should-ice in cfail tests");
            }
            self.run_rpass_test();
        } else if revision.starts_with("rfail") {
            if self.props.should_ice {
                self.fatal("can only use should-ice in cfail tests");
            }
            self.run_rfail_test();
        } else if revision.starts_with("cfail") {
            self.run_cfail_test();
        } else {
            self.fatal("revision name must begin with cpass, rpass, rfail, or cfail");
        }
    }

    fn run_rmake_test(&self) {
        let cwd = env::current_dir().unwrap();
        let src_root = self.config.src_base.parent().unwrap().parent().unwrap();
        let src_root = cwd.join(&src_root);

        let tmpdir = cwd.join(self.output_base_name());
        if tmpdir.exists() {
            self.aggressive_rm_rf(&tmpdir).unwrap();
        }
        create_dir_all(&tmpdir).unwrap();

        let host = &self.config.host;
        let make = if host.contains("dragonfly")
            || host.contains("freebsd")
            || host.contains("netbsd")
            || host.contains("openbsd")
            || host.contains("aix")
        {
            "gmake"
        } else {
            "make"
        };

        let mut cmd = Command::new(make);
        cmd.current_dir(&self.testpaths.file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TARGET", &self.config.target)
            .env("PYTHON", &self.config.python)
            .env("S", src_root)
            .env("RUST_BUILD_STAGE", &self.config.stage_id)
            .env("RUSTC", cwd.join(&self.config.rustc_path))
            .env("TMPDIR", &tmpdir)
            .env("LD_LIB_PATH_ENVVAR", dylib_env_var())
            .env("HOST_RPATH_DIR", cwd.join(&self.config.compile_lib_path))
            .env("TARGET_RPATH_DIR", cwd.join(&self.config.run_lib_path))
            .env("LLVM_COMPONENTS", &self.config.llvm_components)
            // We for sure don't want these tests to run in parallel, so make
            // sure they don't have access to these vars if we run via `make`
            // at the top level
            .env_remove("MAKEFLAGS")
            .env_remove("MFLAGS")
            .env_remove("CARGO_MAKEFLAGS");

        if let Some(ref rustdoc) = self.config.rustdoc_path {
            cmd.env("RUSTDOC", cwd.join(rustdoc));
        }

        if let Some(ref rust_demangler) = self.config.rust_demangler_path {
            cmd.env("RUST_DEMANGLER", cwd.join(rust_demangler));
        }

        if let Some(ref node) = self.config.nodejs {
            cmd.env("NODE", node);
        }

        if let Some(ref linker) = self.config.target_linker {
            cmd.env("RUSTC_LINKER", linker);
        }

        if let Some(ref clang) = self.config.run_clang_based_tests_with {
            cmd.env("CLANG", clang);
        }

        if let Some(ref filecheck) = self.config.llvm_filecheck {
            cmd.env("LLVM_FILECHECK", filecheck);
        }

        if let Some(ref llvm_bin_dir) = self.config.llvm_bin_dir {
            cmd.env("LLVM_BIN_DIR", llvm_bin_dir);
        }

        if let Some(ref remote_test_client) = self.config.remote_test_client {
            cmd.env("REMOTE_TEST_CLIENT", remote_test_client);
        }

        // We don't want RUSTFLAGS set from the outside to interfere with
        // compiler flags set in the test cases:
        cmd.env_remove("RUSTFLAGS");

        // Use dynamic musl for tests because static doesn't allow creating dylibs
        if self.config.host.contains("musl") {
            cmd.env("RUSTFLAGS", "-Ctarget-feature=-crt-static").env("IS_MUSL_HOST", "1");
        }

        if self.config.bless {
            cmd.env("RUSTC_BLESS_TEST", "--bless");
            // Assume this option is active if the environment variable is "defined", with _any_ value.
            // As an example, a `Makefile` can use this option by:
            //
            //   ifdef RUSTC_BLESS_TEST
            //       cp "$(TMPDIR)"/actual_something.ext expected_something.ext
            //   else
            //       $(DIFF) expected_something.ext "$(TMPDIR)"/actual_something.ext
            //   endif
        }

        if self.config.target.contains("msvc") && self.config.cc != "" {
            // We need to pass a path to `lib.exe`, so assume that `cc` is `cl.exe`
            // and that `lib.exe` lives next to it.
            let lib = Path::new(&self.config.cc).parent().unwrap().join("lib.exe");

            // MSYS doesn't like passing flags of the form `/foo` as it thinks it's
            // a path and instead passes `C:\msys64\foo`, so convert all
            // `/`-arguments to MSVC here to `-` arguments.
            let cflags = self
                .config
                .cflags
                .split(' ')
                .map(|s| s.replace("/", "-"))
                .collect::<Vec<_>>()
                .join(" ");
            let cxxflags = self
                .config
                .cxxflags
                .split(' ')
                .map(|s| s.replace("/", "-"))
                .collect::<Vec<_>>()
                .join(" ");

            cmd.env("IS_MSVC", "1")
                .env("IS_WINDOWS", "1")
                .env("MSVC_LIB", format!("'{}' -nologo", lib.display()))
                .env("CC", format!("'{}' {}", self.config.cc, cflags))
                .env("CXX", format!("'{}' {}", &self.config.cxx, cxxflags));
        } else {
            cmd.env("CC", format!("{} {}", self.config.cc, self.config.cflags))
                .env("CXX", format!("{} {}", self.config.cxx, self.config.cxxflags))
                .env("AR", &self.config.ar);

            if self.config.target.contains("windows") {
                cmd.env("IS_WINDOWS", "1");
            }
        }

        let output = self.read2_abbreviated(cmd.spawn().expect("failed to spawn `make`"));
        if !output.status.success() {
            let res = ProcRes {
                status: output.status,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                cmdline: format!("{:?}", cmd),
            };
            self.fatal_proc_rec("make failed", &res);
        }
    }

    fn aggressive_rm_rf(&self, path: &Path) -> io::Result<()> {
        for e in path.read_dir()? {
            let entry = e?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                self.aggressive_rm_rf(&path)?;
            } else {
                // Remove readonly files as well on windows (by default we can't)
                fs::remove_file(&path).or_else(|e| {
                    if cfg!(windows) && e.kind() == io::ErrorKind::PermissionDenied {
                        let mut meta = entry.metadata()?.permissions();
                        meta.set_readonly(false);
                        fs::set_permissions(&path, meta)?;
                        fs::remove_file(&path)
                    } else {
                        Err(e)
                    }
                })?;
            }
        }
        fs::remove_dir(path)
    }

    fn run_js_doc_test(&self) {
        if let Some(nodejs) = &self.config.nodejs {
            let out_dir = self.output_base_dir();

            self.document(&out_dir);

            let root = self.config.find_rust_src_root().unwrap();
            let file_stem =
                self.testpaths.file.file_stem().and_then(|f| f.to_str()).expect("no file stem");
            let res = self.cmd2procres(
                Command::new(&nodejs)
                    .arg(root.join("src/tools/rustdoc-js/tester.js"))
                    .arg("--doc-folder")
                    .arg(out_dir)
                    .arg("--crate-name")
                    .arg(file_stem.replace("-", "_"))
                    .arg("--test-file")
                    .arg(self.testpaths.file.with_extension("js")),
            );
            if !res.status.success() {
                self.fatal_proc_rec("rustdoc-js test failed!", &res);
            }
        } else {
            self.fatal("no nodeJS");
        }
    }

    fn load_compare_outputs(
        &self,
        proc_res: &ProcRes,
        output_kind: TestOutput,
        explicit_format: bool,
    ) -> usize {
        let stderr_bits = format!("{}bit.stderr", self.config.get_pointer_width());
        let (stderr_kind, stdout_kind) = match output_kind {
            TestOutput::Compile => (
                {
                    if self.props.stderr_per_bitwidth { &stderr_bits } else { UI_STDERR }
                },
                UI_STDOUT,
            ),
            TestOutput::Run => (UI_RUN_STDERR, UI_RUN_STDOUT),
        };

        let expected_stderr = self.load_expected_output(stderr_kind);
        let expected_stdout = self.load_expected_output(stdout_kind);

        let normalized_stdout = match output_kind {
            TestOutput::Run if self.config.remote_test_client.is_some() => {
                // When tests are run using the remote-test-client, the string
                // 'uploaded "$TEST_BUILD_DIR/<test_executable>, waiting for result"'
                // is printed to stdout by the client and then captured in the ProcRes,
                // so it needs to be removed when comparing the run-pass test execution output
                static REMOTE_TEST_RE: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(
                        "^uploaded \"\\$TEST_BUILD_DIR(/[[:alnum:]_\\-.]+)+\", waiting for result\n"
                    )
                    .unwrap()
                });
                REMOTE_TEST_RE
                    .replace(
                        &self.normalize_output(&proc_res.stdout, &self.props.normalize_stdout),
                        "",
                    )
                    .to_string()
            }
            _ => self.normalize_output(&proc_res.stdout, &self.props.normalize_stdout),
        };

        let stderr = if explicit_format {
            proc_res.stderr.clone()
        } else {
            json::extract_rendered(&proc_res.stderr)
        };

        let normalized_stderr = self.normalize_output(&stderr, &self.props.normalize_stderr);
        let mut errors = 0;
        match output_kind {
            TestOutput::Compile => {
                if !self.props.dont_check_compiler_stdout {
                    errors += self.compare_output(
                        stdout_kind,
                        &normalized_stdout,
                        &expected_stdout,
                        self.props.compare_output_lines_by_subset,
                    );
                }
                if !self.props.dont_check_compiler_stderr {
                    errors += self.compare_output(
                        stderr_kind,
                        &normalized_stderr,
                        &expected_stderr,
                        self.props.compare_output_lines_by_subset,
                    );
                }
            }
            TestOutput::Run => {
                errors += self.compare_output(
                    stdout_kind,
                    &normalized_stdout,
                    &expected_stdout,
                    self.props.compare_output_lines_by_subset,
                );
                errors += self.compare_output(
                    stderr_kind,
                    &normalized_stderr,
                    &expected_stderr,
                    self.props.compare_output_lines_by_subset,
                );
            }
        }
        errors
    }

    fn run_ui_test(&self) {
        if let Some(FailMode::Build) = self.props.fail_mode {
            // Make sure a build-fail test cannot fail due to failing analysis (e.g. typeck).
            let pm = Some(PassMode::Check);
            let proc_res =
                self.compile_test_general(WillExecute::No, Emit::Metadata, pm, Vec::new());
            self.check_if_test_should_compile(&proc_res, pm);
        }

        let pm = self.pass_mode();
        let should_run = self.should_run(pm);
        let emit_metadata = self.should_emit_metadata(pm);
        let proc_res = self.compile_test(should_run, emit_metadata);
        self.check_if_test_should_compile(&proc_res, pm);

        // if the user specified a format in the ui test
        // print the output to the stderr file, otherwise extract
        // the rendered error messages from json and print them
        let explicit = self.props.compile_flags.iter().any(|s| s.contains("--error-format"));

        let expected_fixed = self.load_expected_output(UI_FIXED);

        self.check_and_prune_duplicate_outputs(&proc_res, &[], &[]);

        let mut errors = self.load_compare_outputs(&proc_res, TestOutput::Compile, explicit);
        let rustfix_input = json::rustfix_diagnostics_only(&proc_res.stderr);

        if self.config.compare_mode.is_some() {
            // don't test rustfix with nll right now
        } else if self.config.rustfix_coverage {
            // Find out which tests have `MachineApplicable` suggestions but are missing
            // `run-rustfix` or `run-rustfix-only-machine-applicable` headers.
            //
            // This will return an empty `Vec` in case the executed test file has a
            // `compile-flags: --error-format=xxxx` header with a value other than `json`.
            let suggestions = get_suggestions_from_json(
                &rustfix_input,
                &HashSet::new(),
                Filter::MachineApplicableOnly,
            )
            .unwrap_or_default();
            if !suggestions.is_empty()
                && !self.props.run_rustfix
                && !self.props.rustfix_only_machine_applicable
            {
                let mut coverage_file_path = self.config.build_base.clone();
                coverage_file_path.push("rustfix_missing_coverage.txt");
                debug!("coverage_file_path: {}", coverage_file_path.display());

                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(coverage_file_path.as_path())
                    .expect("could not create or open file");

                if writeln!(file, "{}", self.testpaths.file.display()).is_err() {
                    panic!("couldn't write to {}", coverage_file_path.display());
                }
            }
        } else if self.props.run_rustfix {
            // Apply suggestions from rustc to the code itself
            let unfixed_code = self.load_expected_output_from_path(&self.testpaths.file).unwrap();
            let suggestions = get_suggestions_from_json(
                &rustfix_input,
                &HashSet::new(),
                if self.props.rustfix_only_machine_applicable {
                    Filter::MachineApplicableOnly
                } else {
                    Filter::Everything
                },
            )
            .unwrap();
            let fixed_code = apply_suggestions(&unfixed_code, &suggestions).unwrap_or_else(|e| {
                panic!(
                    "failed to apply suggestions for {:?} with rustfix: {}",
                    self.testpaths.file, e
                )
            });

            errors += self.compare_output(
                "fixed",
                &fixed_code,
                &expected_fixed,
                self.props.compare_output_lines_by_subset,
            );
        } else if !expected_fixed.is_empty() {
            panic!(
                "the `// run-rustfix` directive wasn't found but a `*.fixed` \
                 file was found"
            );
        }

        if errors > 0 {
            println!("To update references, rerun the tests and pass the `--bless` flag");
            let relative_path_to_file =
                self.testpaths.relative_dir.join(self.testpaths.file.file_name().unwrap());
            println!(
                "To only update this specific test, also pass `--test-args {}`",
                relative_path_to_file.display(),
            );
            self.fatal_proc_rec(
                &format!("{} errors occurred comparing output.", errors),
                &proc_res,
            );
        }

        let expected_errors = errors::load_errors(&self.testpaths.file, self.revision);

        if let WillExecute::Yes = should_run {
            let proc_res = self.exec_compiled_test();
            let run_output_errors = if self.props.check_run_results {
                self.load_compare_outputs(&proc_res, TestOutput::Run, explicit)
            } else {
                0
            };
            if run_output_errors > 0 {
                self.fatal_proc_rec(
                    &format!("{} errors occurred comparing run output.", run_output_errors),
                    &proc_res,
                );
            }
            if self.should_run_successfully(pm) {
                if !proc_res.status.success() {
                    self.fatal_proc_rec("test run failed!", &proc_res);
                }
            } else if proc_res.status.success() {
                self.fatal_proc_rec("test run succeeded!", &proc_res);
            }

            if !self.props.error_patterns.is_empty() || !self.props.regex_error_patterns.is_empty()
            {
                // "// error-pattern" comments
                let output_to_check = self.get_output(&proc_res);
                self.check_all_error_patterns(&output_to_check, &proc_res, pm);
            }
        }

        debug!(
            "run_ui_test: explicit={:?} config.compare_mode={:?} expected_errors={:?} \
               proc_res.status={:?} props.error_patterns={:?}",
            explicit,
            self.config.compare_mode,
            expected_errors,
            proc_res.status,
            self.props.error_patterns
        );
        if !explicit && self.config.compare_mode.is_none() {
            let check_patterns = should_run == WillExecute::No
                && (!self.props.error_patterns.is_empty()
                    || !self.props.regex_error_patterns.is_empty());

            let check_annotations = !check_patterns || !expected_errors.is_empty();

            if check_patterns {
                // "// error-pattern" comments
                let output_to_check = self.get_output(&proc_res);
                self.check_all_error_patterns(&output_to_check, &proc_res, pm);
            }

            if check_annotations {
                // "//~ERROR comments"
                self.check_expected_errors(expected_errors, &proc_res);
            }
        }

        if self.props.run_rustfix && self.config.compare_mode.is_none() {
            // And finally, compile the fixed code and make sure it both
            // succeeds and has no diagnostics.
            let rustc = self.make_compile_args(
                &self.testpaths.file.with_extension(UI_FIXED),
                TargetLocation::ThisFile(self.make_exe_name()),
                emit_metadata,
                AllowUnused::No,
                LinkToAux::Yes,
                Vec::new(),
            );
            let res = self.compose_and_run_compiler(rustc, None);
            if !res.status.success() {
                self.fatal_proc_rec("failed to compile fixed code", &res);
            }
            if !res.stderr.is_empty()
                && !self.props.rustfix_only_machine_applicable
                && !json::rustfix_diagnostics_only(&res.stderr).is_empty()
            {
                self.fatal_proc_rec("fixed code is still producing diagnostics", &res);
            }
        }
    }

    fn run_mir_opt_test(&self) {
        let pm = self.pass_mode();
        let should_run = self.should_run(pm);
        let emit_metadata = self.should_emit_metadata(pm);
        let passes = self.get_passes();

        let proc_res = self.compile_test_with_passes(should_run, emit_metadata, passes);
        self.check_mir_dump();
        if !proc_res.status.success() {
            self.fatal_proc_rec("compilation failed!", &proc_res);
        }

        if let WillExecute::Yes = should_run {
            let proc_res = self.exec_compiled_test();

            if !proc_res.status.success() {
                self.fatal_proc_rec("test run failed!", &proc_res);
            }
        }
    }

    fn get_passes(&self) -> Vec<String> {
        let files = miropt_test_tools::files_for_miropt_test(
            &self.testpaths.file,
            self.config.get_pointer_width(),
            self.config.target_cfg().panic.for_miropt_test_tools(),
        );

        let mut out = Vec::new();

        for miropt_test_tools::MiroptTestFiles {
            from_file: _,
            to_file: _,
            expected_file: _,
            passes,
        } in files
        {
            out.extend(passes);
        }
        out
    }

    fn check_mir_dump(&self) {
        let test_dir = self.testpaths.file.parent().unwrap();
        let test_crate =
            self.testpaths.file.file_stem().unwrap().to_str().unwrap().replace("-", "_");

        let suffix = miropt_test_tools::output_file_suffix(
            &self.testpaths.file,
            self.config.get_pointer_width(),
            self.config.target_cfg().panic.for_miropt_test_tools(),
        );

        if self.config.bless {
            for e in
                glob(&format!("{}/{}.*{}.mir", test_dir.display(), test_crate, suffix)).unwrap()
            {
                std::fs::remove_file(e.unwrap()).unwrap();
            }
            for e in
                glob(&format!("{}/{}.*{}.diff", test_dir.display(), test_crate, suffix)).unwrap()
            {
                std::fs::remove_file(e.unwrap()).unwrap();
            }
        }

        let files = miropt_test_tools::files_for_miropt_test(
            &self.testpaths.file,
            self.config.get_pointer_width(),
            self.config.target_cfg().panic.for_miropt_test_tools(),
        );
        for miropt_test_tools::MiroptTestFiles { from_file, to_file, expected_file, passes: _ } in
            files
        {
            let dumped_string = if let Some(after) = to_file {
                self.diff_mir_files(from_file.into(), after.into())
            } else {
                let mut output_file = PathBuf::new();
                output_file.push(self.get_mir_dump_dir());
                output_file.push(&from_file);
                debug!(
                    "comparing the contents of: {} with {}",
                    output_file.display(),
                    expected_file.display()
                );
                if !output_file.exists() {
                    panic!(
                        "Output file `{}` from test does not exist, available files are in `{}`",
                        output_file.display(),
                        output_file.parent().unwrap().display()
                    );
                }
                self.check_mir_test_timestamp(&from_file, &output_file);
                let dumped_string = fs::read_to_string(&output_file).unwrap();
                self.normalize_output(&dumped_string, &[])
            };

            if self.config.bless {
                let _ = std::fs::remove_file(&expected_file);
                std::fs::write(expected_file, dumped_string.as_bytes()).unwrap();
            } else {
                if !expected_file.exists() {
                    panic!("Output file `{}` from test does not exist", expected_file.display());
                }
                let expected_string = fs::read_to_string(&expected_file).unwrap();
                if dumped_string != expected_string {
                    print!("{}", write_diff(&expected_string, &dumped_string, 3));
                    panic!(
                        "Actual MIR output differs from expected MIR output {}",
                        expected_file.display()
                    );
                }
            }
        }
    }

    fn diff_mir_files(&self, before: PathBuf, after: PathBuf) -> String {
        let to_full_path = |path: PathBuf| {
            let full = self.get_mir_dump_dir().join(&path);
            if !full.exists() {
                panic!(
                    "the mir dump file for {} does not exist (requested in {})",
                    path.display(),
                    self.testpaths.file.display(),
                );
            }
            full
        };
        let before = to_full_path(before);
        let after = to_full_path(after);
        debug!("comparing the contents of: {} with {}", before.display(), after.display());
        let before = fs::read_to_string(before).unwrap();
        let after = fs::read_to_string(after).unwrap();
        let before = self.normalize_output(&before, &[]);
        let after = self.normalize_output(&after, &[]);
        let mut dumped_string = String::new();
        for result in diff::lines(&before, &after) {
            use std::fmt::Write;
            match result {
                diff::Result::Left(s) => writeln!(dumped_string, "- {}", s).unwrap(),
                diff::Result::Right(s) => writeln!(dumped_string, "+ {}", s).unwrap(),
                diff::Result::Both(s, _) => writeln!(dumped_string, "  {}", s).unwrap(),
            }
        }
        dumped_string
    }

    fn check_mir_test_timestamp(&self, test_name: &str, output_file: &Path) {
        let t = |file| fs::metadata(file).unwrap().modified().unwrap();
        let source_file = &self.testpaths.file;
        let output_time = t(output_file);
        let source_time = t(source_file);
        if source_time > output_time {
            debug!("source file time: {:?} output file time: {:?}", source_time, output_time);
            panic!(
                "test source file `{}` is newer than potentially stale output file `{}`.",
                source_file.display(),
                test_name
            );
        }
    }

    fn get_mir_dump_dir(&self) -> PathBuf {
        let mut mir_dump_dir = PathBuf::from(self.config.build_base.as_path());
        debug!("input_file: {:?}", self.testpaths.file);
        mir_dump_dir.push(&self.testpaths.relative_dir);
        mir_dump_dir.push(self.testpaths.file.file_stem().unwrap());
        mir_dump_dir
    }

    fn normalize_output(&self, output: &str, custom_rules: &[(String, String)]) -> String {
        let rflags = self.props.run_flags.as_ref();
        let cflags = self.props.compile_flags.join(" ");
        let json = rflags
            .map_or(false, |s| s.contains("--format json") || s.contains("--format=json"))
            || cflags.contains("--error-format json")
            || cflags.contains("--error-format pretty-json")
            || cflags.contains("--error-format=json")
            || cflags.contains("--error-format=pretty-json")
            || cflags.contains("--output-format json")
            || cflags.contains("--output-format=json");

        let mut normalized = output.to_string();

        let mut normalize_path = |from: &Path, to: &str| {
            let mut from = from.display().to_string();
            if json {
                from = from.replace("\\", "\\\\");
            }
            normalized = normalized.replace(&from, to);
        };

        let parent_dir = self.testpaths.file.parent().unwrap();
        normalize_path(parent_dir, "$DIR");

        if self.props.remap_src_base {
            let mut remapped_parent_dir = PathBuf::from(FAKE_SRC_BASE);
            if self.testpaths.relative_dir != Path::new("") {
                remapped_parent_dir.push(&self.testpaths.relative_dir);
            }
            normalize_path(&remapped_parent_dir, "$DIR");
        }

        let base_dir = Path::new("/rustc/FAKE_PREFIX");
        // Paths into the libstd/libcore
        normalize_path(&base_dir.join("library"), "$SRC_DIR");
        // `ui-fulldeps` tests can show paths to the compiler source when testing macros from
        // `rustc_macros`
        // eg. /home/user/rust/compiler
        normalize_path(&base_dir.join("compiler"), "$COMPILER_DIR");

        // Paths into the build directory
        let test_build_dir = &self.config.build_base;
        let parent_build_dir = test_build_dir.parent().unwrap().parent().unwrap().parent().unwrap();

        // eg. /home/user/rust/build/x86_64-unknown-linux-gnu/test/ui
        normalize_path(test_build_dir, "$TEST_BUILD_DIR");
        // eg. /home/user/rust/build
        normalize_path(parent_build_dir, "$BUILD_DIR");

        // Paths into lib directory.
        normalize_path(&parent_build_dir.parent().unwrap().join("lib"), "$LIB_DIR");

        if json {
            // escaped newlines in json strings should be readable
            // in the stderr files. There's no point int being correct,
            // since only humans process the stderr files.
            // Thus we just turn escaped newlines back into newlines.
            normalized = normalized.replace("\\n", "\n");
        }

        // If there are `$SRC_DIR` normalizations with line and column numbers, then replace them
        // with placeholders as we do not want tests needing updated when compiler source code
        // changes.
        // eg. $SRC_DIR/libcore/mem.rs:323:14 becomes $SRC_DIR/libcore/mem.rs:LL:COL
        static SRC_DIR_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new("SRC_DIR(.+):\\d+:\\d+(: \\d+:\\d+)?").unwrap());

        normalized = SRC_DIR_RE.replace_all(&normalized, "SRC_DIR$1:LL:COL").into_owned();

        normalized = Self::normalize_platform_differences(&normalized);
        normalized = normalized.replace("\t", "\\t"); // makes tabs visible

        // Remove test annotations like `//~ ERROR text` from the output,
        // since they duplicate actual errors and make the output hard to read.
        // This mirrors the regex in src/tools/tidy/src/style.rs, please update
        // both if either are changed.
        static ANNOTATION_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new("\\s*//(\\[.*\\])?~.*").unwrap());

        normalized = ANNOTATION_RE.replace_all(&normalized, "").into_owned();

        // This code normalizes various hashes in v0 symbol mangling that is
        // emitted in the ui and mir-opt tests.
        static V0_CRATE_HASH_PREFIX_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"_R.*?Cs[0-9a-zA-Z]+_").unwrap());
        static V0_CRATE_HASH_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"Cs[0-9a-zA-Z]+_").unwrap());

        const V0_CRATE_HASH_PLACEHOLDER: &str = r"CsCRATE_HASH_";
        if V0_CRATE_HASH_PREFIX_RE.is_match(&normalized) {
            // Normalize crate hash
            normalized =
                V0_CRATE_HASH_RE.replace_all(&normalized, V0_CRATE_HASH_PLACEHOLDER).into_owned();
        }

        static V0_BACK_REF_PREFIX_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\(_R.*?B[0-9a-zA-Z]_").unwrap());
        static V0_BACK_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"B[0-9a-zA-Z]_").unwrap());

        const V0_BACK_REF_PLACEHOLDER: &str = r"B<REF>_";
        if V0_BACK_REF_PREFIX_RE.is_match(&normalized) {
            // Normalize back references (see RFC 2603)
            normalized =
                V0_BACK_REF_RE.replace_all(&normalized, V0_BACK_REF_PLACEHOLDER).into_owned();
        }

        // Custom normalization rules
        for rule in custom_rules {
            let re = Regex::new(&rule.0).expect("bad regex in custom normalization rule");
            normalized = re.replace_all(&normalized, &rule.1[..]).into_owned();
        }
        normalized
    }

    /// Normalize output differences across platforms. Generally changes Windows output to be more
    /// Unix-like.
    ///
    /// Replaces backslashes in paths with forward slashes, and replaces CRLF line endings
    /// with LF.
    fn normalize_platform_differences(output: &str) -> String {
        /// Used to find Windows paths.
        ///
        /// It's not possible to detect paths in the error messages generally, but this is a
        /// decent enough heuristic.
        static PATH_BACKSLASH_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r#"(?x)
                (?:
                  # Match paths that don't include spaces.
                  (?:\\[\pL\pN\.\-_']+)+\.\pL+
                |
                  # If the path starts with a well-known root, then allow spaces and no file extension.
                  \$(?:DIR|SRC_DIR|TEST_BUILD_DIR|BUILD_DIR|LIB_DIR)(?:\\[\pL\pN\.\-_'\ ]+)+
                )"#,
            )
            .unwrap()
        });

        let output = output.replace(r"\\", r"\");

        PATH_BACKSLASH_RE
            .replace_all(&output, |caps: &Captures<'_>| {
                println!("{}", &caps[0]);
                caps[0].replace(r"\", "/")
            })
            .replace("\r\n", "\n")
    }

    fn expected_output_path(&self, kind: &str) -> PathBuf {
        let mut path =
            expected_output_path(&self.testpaths, self.revision, &self.config.compare_mode, kind);

        if !path.exists() {
            if let Some(CompareMode::Polonius) = self.config.compare_mode {
                path = expected_output_path(&self.testpaths, self.revision, &None, kind);
            }
        }

        if !path.exists() {
            path = expected_output_path(&self.testpaths, self.revision, &None, kind);
        }

        path
    }

    fn load_expected_output(&self, kind: &str) -> String {
        let path = self.expected_output_path(kind);
        if path.exists() {
            match self.load_expected_output_from_path(&path) {
                Ok(x) => x,
                Err(x) => self.fatal(&x),
            }
        } else {
            String::new()
        }
    }

    fn load_expected_output_from_path(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| {
            format!("failed to load expected output from `{}`: {}", path.display(), err)
        })
    }

    fn delete_file(&self, file: &PathBuf) {
        if !file.exists() {
            // Deleting a nonexistent file would error.
            return;
        }
        if let Err(e) = fs::remove_file(file) {
            self.fatal(&format!("failed to delete `{}`: {}", file.display(), e,));
        }
    }

    fn compare_output(
        &self,
        kind: &str,
        actual: &str,
        expected: &str,
        compare_output_by_lines: bool,
    ) -> usize {
        if actual == expected {
            return 0;
        }

        let tmp;
        let (expected, actual): (&str, &str) = if compare_output_by_lines {
            let actual_lines: HashSet<_> = actual.lines().collect();
            let expected_lines: Vec<_> = expected.lines().collect();
            let mut used = expected_lines.clone();
            used.retain(|line| actual_lines.contains(line));
            // check if `expected` contains a subset of the lines of `actual`
            if used.len() == expected_lines.len() && (expected.is_empty() == actual.is_empty()) {
                return 0;
            }
            if expected_lines.is_empty() {
                // if we have no lines to check, force a full overwite
                ("", actual)
            } else {
                tmp = (expected_lines.join("\n"), used.join("\n"));
                (&tmp.0, &tmp.1)
            }
        } else {
            (expected, actual)
        };

        if !self.config.bless {
            if expected.is_empty() {
                println!("normalized {}:\n{}\n", kind, actual);
            } else {
                println!("diff of {}:\n", kind);
                print!("{}", write_diff(expected, actual, 3));
            }
        }

        let mode = self.config.compare_mode.as_ref().map_or("", |m| m.to_str());
        let output_file = self
            .output_base_name()
            .with_extra_extension(self.revision.unwrap_or(""))
            .with_extra_extension(mode)
            .with_extra_extension(kind);

        let mut files = vec![output_file];
        if self.config.bless {
            // Delete non-revision .stderr/.stdout file if revisions are used.
            // Without this, we'd just generate the new files and leave the old files around.
            if self.revision.is_some() {
                let old =
                    expected_output_path(self.testpaths, None, &self.config.compare_mode, kind);
                self.delete_file(&old);
            }
            files.push(expected_output_path(
                self.testpaths,
                self.revision,
                &self.config.compare_mode,
                kind,
            ));
        }

        for output_file in &files {
            if actual.is_empty() {
                self.delete_file(output_file);
            } else if let Err(err) = fs::write(&output_file, &actual) {
                self.fatal(&format!(
                    "failed to write {} to `{}`: {}",
                    kind,
                    output_file.display(),
                    err,
                ));
            }
        }

        println!("\nThe actual {0} differed from the expected {0}.", kind);
        for output_file in files {
            println!("Actual {} saved to {}", kind, output_file.display());
        }
        if self.config.bless { 0 } else { 1 }
    }

    fn check_and_prune_duplicate_outputs(
        &self,
        proc_res: &ProcRes,
        modes: &[CompareMode],
        require_same_modes: &[CompareMode],
    ) {
        for kind in UI_EXTENSIONS {
            let canon_comparison_path =
                expected_output_path(&self.testpaths, self.revision, &None, kind);

            let canon = match self.load_expected_output_from_path(&canon_comparison_path) {
                Ok(canon) => canon,
                _ => continue,
            };
            let bless = self.config.bless;
            let check_and_prune_duplicate_outputs = |mode: &CompareMode, require_same: bool| {
                let examined_path =
                    expected_output_path(&self.testpaths, self.revision, &Some(mode.clone()), kind);

                // If there is no output, there is nothing to do
                let examined_content = match self.load_expected_output_from_path(&examined_path) {
                    Ok(content) => content,
                    _ => return,
                };

                let is_duplicate = canon == examined_content;

                match (bless, require_same, is_duplicate) {
                    // If we're blessing and the output is the same, then delete the file.
                    (true, _, true) => {
                        self.delete_file(&examined_path);
                    }
                    // If we want them to be the same, but they are different, then error.
                    // We do this wether we bless or not
                    (_, true, false) => {
                        self.fatal_proc_rec(
                            &format!("`{}` should not have different output from base test!", kind),
                            proc_res,
                        );
                    }
                    _ => {}
                }
            };
            for mode in modes {
                check_and_prune_duplicate_outputs(mode, false);
            }
            for mode in require_same_modes {
                check_and_prune_duplicate_outputs(mode, true);
            }
        }
    }

    fn create_stamp(&self) {
        let stamp = crate::stamp(&self.config, self.testpaths, self.revision);
        fs::write(&stamp, compute_stamp_hash(&self.config)).unwrap();
    }
}

struct ProcArgs {
    prog: String,
    args: Vec<String>,
}

pub struct ProcRes {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    cmdline: String,
}

impl ProcRes {
    pub fn print_info(&self) {
        fn render(name: &str, contents: &str) -> String {
            let contents = json::extract_rendered(contents);
            let contents = contents.trim_end();
            if contents.is_empty() {
                format!("{name}: none")
            } else {
                format!(
                    "\
                     --- {name} -------------------------------\n\
                     {contents}\n\
                     ------------------------------------------",
                )
            }
        }

        println!(
            "status: {}\ncommand: {}\n{}\n{}\n",
            self.status,
            self.cmdline,
            render("stdout", &self.stdout),
            render("stderr", &self.stderr),
        );
    }

    pub fn fatal(&self, err: Option<&str>, on_failure: impl FnOnce()) -> ! {
        if let Some(e) = err {
            println!("\nerror: {}", e);
        }
        self.print_info();
        on_failure();
        // Use resume_unwind instead of panic!() to prevent a panic message + backtrace from
        // compiletest, which is unnecessary noise.
        std::panic::resume_unwind(Box::new(()));
    }
}

#[derive(Debug)]
enum TargetLocation {
    ThisFile(PathBuf),
    ThisDirectory(PathBuf),
}

enum AllowUnused {
    Yes,
    No,
}

enum LinkToAux {
    Yes,
    No,
}
