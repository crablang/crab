//! Implementation of the test-related targets of the build system.
//!
//! This file implements the various regression test suites that we execute on
//! our CI.

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::builder::crate_description;
use crate::builder::{Builder, Compiler, Kind, RunConfig, ShouldRun, Step};
use crate::cache::Interned;
use crate::compile;
use crate::config::TargetSelection;
use crate::dist;
use crate::doc::DocumentationFormat;
use crate::flags::Subcommand;
use crate::llvm;
use crate::render_tests::add_flags_and_try_run_tests;
use crate::tool::{self, SourceType, Tool};
use crate::toolstate::ToolState;
use crate::util::{self, add_link_lib_path, dylib_path, dylib_path_var, output, t, up_to_date};
use crate::{envify, CLang, DocTests, GitRepo, Mode};

const ADB_TEST_DIR: &str = "/data/local/tmp/work";

/// The two modes of the test runner; tests or benchmarks.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum TestKind {
    /// Run `cargo test`.
    Test,
    /// Run `cargo bench`.
    Bench,
}

impl From<Kind> for TestKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Test => TestKind::Test,
            Kind::Bench => TestKind::Bench,
            _ => panic!("unexpected kind in crate: {:?}", kind),
        }
    }
}

impl TestKind {
    // Return the cargo subcommand for this test kind
    fn subcommand(self) -> &'static str {
        match self {
            TestKind::Test => "test",
            TestKind::Bench => "bench",
        }
    }
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            TestKind::Test => "Testing",
            TestKind::Bench => "Benchmarking",
        })
    }
}

fn try_run(builder: &Builder<'_>, cmd: &mut Command) -> bool {
    if !builder.fail_fast {
        if !builder.try_run(cmd) {
            let mut failures = builder.delayed_failures.borrow_mut();
            failures.push(format!("{:?}", cmd));
            return false;
        }
    } else {
        builder.run(cmd);
    }
    true
}

fn try_run_quiet(builder: &Builder<'_>, cmd: &mut Command) -> bool {
    if !builder.fail_fast {
        if !builder.try_run_quiet(cmd) {
            let mut failures = builder.delayed_failures.borrow_mut();
            failures.push(format!("{:?}", cmd));
            return false;
        }
    } else {
        builder.run_quiet(cmd);
    }
    true
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrateJsonDocLint {
    host: TargetSelection,
}

impl Step for CrateJsonDocLint {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/jsondoclint")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrateJsonDocLint { host: run.target });
    }

    fn run(self, builder: &Builder<'_>) {
        let bootstrap_host = builder.config.build;
        let compiler = builder.compiler(0, bootstrap_host);

        let cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolBootstrap,
            bootstrap_host,
            "test",
            "src/tools/jsondoclint",
            SourceType::InTree,
            &[],
        );
        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Linkcheck {
    host: TargetSelection,
}

impl Step for Linkcheck {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    /// Runs the `linkchecker` tool as compiled in `stage` by the `host` compiler.
    ///
    /// This tool in `src/tools` will verify the validity of all our links in the
    /// documentation to ensure we don't have a bunch of dead ones.
    fn run(self, builder: &Builder<'_>) {
        let host = self.host;
        let hosts = &builder.hosts;
        let targets = &builder.targets;

        // if we have different hosts and targets, some things may be built for
        // the host (e.g. crablangc) and others for the target (e.g. std). The
        // documentation built for each will contain broken links to
        // docs built for the other platform (e.g. crablangc linking to cargo)
        if (hosts != targets) && !hosts.is_empty() && !targets.is_empty() {
            panic!(
                "Linkcheck currently does not support builds with different hosts and targets.
You can skip linkcheck with --exclude src/tools/linkchecker"
            );
        }

        builder.info(&format!("Linkcheck ({})", host));

        // Test the linkchecker itself.
        let bootstrap_host = builder.config.build;
        let compiler = builder.compiler(0, bootstrap_host);
        let cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolBootstrap,
            bootstrap_host,
            "test",
            "src/tools/linkchecker",
            SourceType::InTree,
            &[],
        );
        add_flags_and_try_run_tests(builder, &mut cargo.into());

        // Build all the default documentation.
        builder.default_doc(&[]);

        // Run the linkchecker.
        let _time = util::timeit(&builder);
        try_run(
            builder,
            builder.tool_cmd(Tool::Linkchecker).arg(builder.out.join(host.triple).join("doc")),
        );
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        let run = run.path("src/tools/linkchecker");
        run.default_condition(builder.config.docs)
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Linkcheck { host: run.target });
    }
}

fn check_if_tidy_is_installed() -> bool {
    Command::new("tidy")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .map_or(false, |status| status.success())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HtmlCheck {
    target: TargetSelection,
}

impl Step for HtmlCheck {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let run = run.path("src/tools/html-checker");
        run.lazy_default_condition(Box::new(check_if_tidy_is_installed))
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(HtmlCheck { target: run.target });
    }

    fn run(self, builder: &Builder<'_>) {
        if !check_if_tidy_is_installed() {
            eprintln!("not running HTML-check tool because `tidy` is missing");
            eprintln!(
                "Note that `tidy` is not the in-tree `src/tools/tidy` but needs to be installed"
            );
            panic!("Cannot run html-check tests");
        }
        // Ensure that a few different kinds of documentation are available.
        builder.default_doc(&[]);
        builder.ensure(crate::doc::CrabLangc { target: self.target, stage: builder.top_stage });

        try_run(builder, builder.tool_cmd(Tool::HtmlChecker).arg(builder.doc_out(self.target)));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cargotest {
    stage: u32,
    host: TargetSelection,
}

impl Step for Cargotest {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/cargotest")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Cargotest { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs the `cargotest` tool as compiled in `stage` by the `host` compiler.
    ///
    /// This tool in `src/tools` will check out a few CrabLang projects and run `cargo
    /// test` to ensure that we don't regress the test suites there.
    fn run(self, builder: &Builder<'_>) {
        let compiler = builder.compiler(self.stage, self.host);
        builder.ensure(compile::CrabLangc::new(compiler, compiler.host));
        let cargo = builder.ensure(tool::Cargo { compiler, target: compiler.host });

        // Note that this is a short, cryptic, and not scoped directory name. This
        // is currently to minimize the length of path on Windows where we otherwise
        // quickly run into path name limit constraints.
        let out_dir = builder.out.join("ct");
        t!(fs::create_dir_all(&out_dir));

        let _time = util::timeit(&builder);
        let mut cmd = builder.tool_cmd(Tool::CargoTest);
        try_run(
            builder,
            cmd.arg(&cargo)
                .arg(&out_dir)
                .args(builder.config.cmd.test_args())
                .env("CRABLANGC", builder.crablangc(compiler))
                .env("CRABLANGDOC", builder.crablangdoc(compiler)),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cargo {
    stage: u32,
    host: TargetSelection,
}

impl Step for Cargo {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/cargo")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Cargo { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs `cargo test` for `cargo` packaged with CrabLang.
    fn run(self, builder: &Builder<'_>) {
        let compiler = builder.compiler(self.stage, self.host);

        builder.ensure(tool::Cargo { compiler, target: self.host });
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            self.host,
            "test",
            "src/tools/cargo",
            SourceType::Submodule,
            &[],
        );

        if !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }
        cargo.arg("--").args(builder.config.cmd.test_args());

        // Don't run cross-compile tests, we may not have cross-compiled libstd libs
        // available.
        cargo.env("CFG_DISABLE_CROSS_TESTS", "1");
        // Forcibly disable tests using nightly features since any changes to
        // those features won't be able to land.
        cargo.env("CARGO_TEST_DISABLE_NIGHTLY", "1");

        cargo.env("PATH", &path_for_cargo(builder, compiler));

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangAnalyzer {
    stage: u32,
    host: TargetSelection,
}

impl Step for CrabLangAnalyzer {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablang-analyzer")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Self { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs `cargo test` for crablang-analyzer
    fn run(self, builder: &Builder<'_>) {
        let stage = self.stage;
        let host = self.host;
        let compiler = builder.compiler(stage, host);

        builder.ensure(tool::CrabLangAnalyzer { compiler, target: self.host }).expect("in-tree tool");

        let workspace_path = "src/tools/crablang-analyzer";
        // until the whole RA test suite runs on `i686`, we only run
        // `proc-macro-srv` tests
        let crate_path = "src/tools/crablang-analyzer/crates/proc-macro-srv";
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolStd,
            host,
            "test",
            crate_path,
            SourceType::InTree,
            &["sysroot-abi".to_owned()],
        );
        cargo.allow_features(tool::CrabLangAnalyzer::ALLOW_FEATURES);

        let dir = builder.src.join(workspace_path);
        // needed by crablang-analyzer to find its own text fixtures, cf.
        // https://github.com/crablang-analyzer/expect-test/issues/33
        cargo.env("CARGO_WORKSPACE_DIR", &dir);

        // RA's test suite tries to write to the source directory, that can't
        // work in CrabLang CI
        cargo.env("SKIP_SLOW_TESTS", "1");

        cargo.add_crablangc_lib_path(builder, compiler);
        cargo.arg("--").args(builder.config.cmd.test_args());

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangfmt {
    stage: u32,
    host: TargetSelection,
}

impl Step for CrabLangfmt {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablangfmt")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangfmt { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs `cargo test` for crablangfmt.
    fn run(self, builder: &Builder<'_>) {
        let stage = self.stage;
        let host = self.host;
        let compiler = builder.compiler(stage, host);

        builder
            .ensure(tool::CrabLangfmt { compiler, target: self.host, extra_features: Vec::new() })
            .expect("in-tree tool");

        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            host,
            "test",
            "src/tools/crablangfmt",
            SourceType::InTree,
            &[],
        );

        if !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }

        let dir = testdir(builder, compiler.host);
        t!(fs::create_dir_all(&dir));
        cargo.env("CRABLANGFMT_TEST_DIR", dir);

        cargo.add_crablangc_lib_path(builder, compiler);

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangDemangler {
    stage: u32,
    host: TargetSelection,
}

impl Step for CrabLangDemangler {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablang-demangler")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangDemangler { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs `cargo test` for crablang-demangler.
    fn run(self, builder: &Builder<'_>) {
        let stage = self.stage;
        let host = self.host;
        let compiler = builder.compiler(stage, host);

        let crablang_demangler = builder
            .ensure(tool::CrabLangDemangler { compiler, target: self.host, extra_features: Vec::new() })
            .expect("in-tree tool");
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            host,
            "test",
            "src/tools/crablang-demangler",
            SourceType::InTree,
            &[],
        );

        let dir = testdir(builder, compiler.host);
        t!(fs::create_dir_all(&dir));

        cargo.env("CRABLANG_DEMANGLER_DRIVER_PATH", crablang_demangler);

        cargo.arg("--").args(builder.config.cmd.test_args());

        cargo.add_crablangc_lib_path(builder, compiler);

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Miri {
    stage: u32,
    host: TargetSelection,
    target: TargetSelection,
}

impl Miri {
    /// Run `cargo miri setup` for the given target, return where the Miri sysroot was put.
    pub fn build_miri_sysroot(
        builder: &Builder<'_>,
        compiler: Compiler,
        miri: &Path,
        target: TargetSelection,
    ) -> String {
        let miri_sysroot = builder.out.join(compiler.host.triple).join("miri-sysroot");
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            compiler.host,
            "run",
            "src/tools/miri/cargo-miri",
            SourceType::InTree,
            &[],
        );
        cargo.add_crablangc_lib_path(builder, compiler);
        cargo.arg("--").arg("miri").arg("setup");
        cargo.arg("--target").arg(target.crablangc_target_arg());

        // Tell `cargo miri setup` where to find the sources.
        cargo.env("MIRI_LIB_SRC", builder.src.join("library"));
        // Tell it where to find Miri.
        cargo.env("MIRI", &miri);
        // Tell it where to put the sysroot.
        cargo.env("MIRI_SYSROOT", &miri_sysroot);
        // Debug things.
        cargo.env("CRABLANG_BACKTRACE", "1");

        let mut cargo = Command::from(cargo);
        builder.run(&mut cargo);

        // # Determine where Miri put its sysroot.
        // To this end, we run `cargo miri setup --print-sysroot` and capture the output.
        // (We do this separately from the above so that when the setup actually
        // happens we get some output.)
        // We re-use the `cargo` from above.
        cargo.arg("--print-sysroot");

        // FIXME: Is there a way in which we can re-use the usual `run` helpers?
        if builder.config.dry_run() {
            String::new()
        } else {
            builder.verbose(&format!("running: {:?}", cargo));
            let out =
                cargo.output().expect("We already ran `cargo miri setup` before and that worked");
            assert!(out.status.success(), "`cargo miri setup` returned with non-0 exit code");
            // Output is "<sysroot>\n".
            let stdout = String::from_utf8(out.stdout)
                .expect("`cargo miri setup` stdout is not valid UTF-8");
            let sysroot = stdout.trim_end();
            builder.verbose(&format!("`cargo miri setup --print-sysroot` said: {:?}", sysroot));
            sysroot.to_owned()
        }
    }
}

impl Step for Miri {
    type Output = ();
    const ONLY_HOSTS: bool = false;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/miri")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Miri {
            stage: run.builder.top_stage,
            host: run.build_triple(),
            target: run.target,
        });
    }

    /// Runs `cargo test` for miri.
    fn run(self, builder: &Builder<'_>) {
        let stage = self.stage;
        let host = self.host;
        let target = self.target;
        let compiler = builder.compiler(stage, host);
        // We need the stdlib for the *next* stage, as it was built with this compiler that also built Miri.
        // Except if we are at stage 2, the bootstrap loop is complete and we can stick with our current stage.
        let compiler_std = builder.compiler(if stage < 2 { stage + 1 } else { stage }, host);

        let miri = builder
            .ensure(tool::Miri { compiler, target: self.host, extra_features: Vec::new() })
            .expect("in-tree tool");
        let _cargo_miri = builder
            .ensure(tool::CargoMiri { compiler, target: self.host, extra_features: Vec::new() })
            .expect("in-tree tool");
        // The stdlib we need might be at a different stage. And just asking for the
        // sysroot does not seem to populate it, so we do that first.
        builder.ensure(compile::Std::new(compiler_std, host));
        let sysroot = builder.sysroot(compiler_std);
        // We also need a Miri sysroot.
        let miri_sysroot = Miri::build_miri_sysroot(builder, compiler, &miri, target);

        // # Run `cargo test`.
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            host,
            "test",
            "src/tools/miri",
            SourceType::InTree,
            &[],
        );
        cargo.add_crablangc_lib_path(builder, compiler);

        if !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }

        // miri tests need to know about the stage sysroot
        cargo.env("MIRI_SYSROOT", &miri_sysroot);
        cargo.env("MIRI_HOST_SYSROOT", sysroot);
        cargo.env("MIRI", &miri);
        // propagate --bless
        if builder.config.cmd.bless() {
            cargo.env("MIRI_BLESS", "Gesundheit");
        }

        // Set the target.
        cargo.env("MIRI_TEST_TARGET", target.crablangc_target_arg());
        // Forward test filters.
        cargo.arg("--").args(builder.config.cmd.test_args());

        add_flags_and_try_run_tests(builder, &mut cargo.into());

        // # Run `cargo miri test`.
        // This is just a smoke test (Miri's own CI invokes this in a bunch of different ways and ensures
        // that we get the desired output), but that is sufficient to make sure that the libtest harness
        // itself executes properly under Miri.
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            host,
            "run",
            "src/tools/miri/cargo-miri",
            SourceType::Submodule,
            &[],
        );
        cargo.add_crablangc_lib_path(builder, compiler);
        cargo.arg("--").arg("miri").arg("test");
        cargo
            .arg("--manifest-path")
            .arg(builder.src.join("src/tools/miri/test-cargo-miri/Cargo.toml"));
        cargo.arg("--target").arg(target.crablangc_target_arg());
        cargo.arg("--tests"); // don't run doctests, they are too confused by the staging
        cargo.arg("--").args(builder.config.cmd.test_args());

        // Tell `cargo miri` where to find things.
        cargo.env("MIRI_SYSROOT", &miri_sysroot);
        cargo.env("MIRI_HOST_SYSROOT", sysroot);
        cargo.env("MIRI", &miri);
        // Debug things.
        cargo.env("CRABLANG_BACKTRACE", "1");

        let mut cargo = Command::from(cargo);
        builder.run(&mut cargo);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CompiletestTest {
    host: TargetSelection,
}

impl Step for CompiletestTest {
    type Output = ();

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/compiletest")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CompiletestTest { host: run.target });
    }

    /// Runs `cargo test` for compiletest.
    fn run(self, builder: &Builder<'_>) {
        let host = self.host;
        let compiler = builder.compiler(1, host);

        // We need `ToolStd` for the locally-built sysroot because
        // compiletest uses unstable features of the `test` crate.
        builder.ensure(compile::Std::new(compiler, host));
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolStd,
            host,
            "test",
            "src/tools/compiletest",
            SourceType::InTree,
            &[],
        );
        cargo.allow_features("test");

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Clippy {
    stage: u32,
    host: TargetSelection,
}

impl Step for Clippy {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = false;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/clippy")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Clippy { stage: run.builder.top_stage, host: run.target });
    }

    /// Runs `cargo test` for clippy.
    fn run(self, builder: &Builder<'_>) {
        let stage = self.stage;
        let host = self.host;
        let compiler = builder.compiler(stage, host);

        builder
            .ensure(tool::Clippy { compiler, target: self.host, extra_features: Vec::new() })
            .expect("in-tree tool");
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            host,
            "test",
            "src/tools/clippy",
            SourceType::InTree,
            &[],
        );

        if !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }

        cargo.env("CRABLANGC_TEST_SUITE", builder.crablangc(compiler));
        cargo.env("CRABLANGC_LIB_PATH", builder.crablangc_libdir(compiler));
        let host_libs = builder.stage_out(compiler, Mode::ToolCrabLangc).join(builder.cargo_dir());
        cargo.env("HOST_LIBS", host_libs);

        cargo.arg("--").args(builder.config.cmd.test_args());

        cargo.add_crablangc_lib_path(builder, compiler);

        if builder.try_run(&mut cargo.into()) {
            // The tests succeeded; nothing to do.
            return;
        }

        if !builder.config.cmd.bless() {
            crate::detail_exit(1);
        }

        let mut cargo = builder.cargo(compiler, Mode::ToolCrabLangc, SourceType::InTree, host, "run");
        cargo.arg("-p").arg("clippy_dev");
        // clippy_dev gets confused if it can't find `clippy/Cargo.toml`
        cargo.current_dir(&builder.src.join("src").join("tools").join("clippy"));
        if builder.config.crablang_optimize {
            cargo.env("PROFILE", "release");
        } else {
            cargo.env("PROFILE", "debug");
        }
        cargo.arg("--");
        cargo.arg("bless");
        builder.run(&mut cargo.into());
    }
}

fn path_for_cargo(builder: &Builder<'_>, compiler: Compiler) -> OsString {
    // Configure PATH to find the right crablangc. NB. we have to use PATH
    // and not CRABLANGC because the Cargo test suite has tests that will
    // fail if crablangc is not spelled `crablangc`.
    let path = builder.sysroot(compiler).join("bin");
    let old_path = env::var_os("PATH").unwrap_or_default();
    env::join_paths(iter::once(path).chain(env::split_paths(&old_path))).expect("")
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangdocTheme {
    pub compiler: Compiler,
}

impl Step for CrabLangdocTheme {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablangdoc-themes")
    }

    fn make_run(run: RunConfig<'_>) {
        let compiler = run.builder.compiler(run.builder.top_stage, run.target);

        run.builder.ensure(CrabLangdocTheme { compiler });
    }

    fn run(self, builder: &Builder<'_>) {
        let crablangdoc = builder.bootstrap_out.join("crablangdoc");
        let mut cmd = builder.tool_cmd(Tool::CrabLangdocTheme);
        cmd.arg(crablangdoc.to_str().unwrap())
            .arg(builder.src.join("src/libcrablangdoc/html/static/css/themes").to_str().unwrap())
            .env("CRABLANGC_STAGE", self.compiler.stage.to_string())
            .env("CRABLANGC_SYSROOT", builder.sysroot(self.compiler))
            .env("CRABLANGDOC_LIBDIR", builder.sysroot_libdir(self.compiler, self.compiler.host))
            .env("CFG_RELEASE_CHANNEL", &builder.config.channel)
            .env("CRABLANGDOC_REAL", builder.crablangdoc(self.compiler))
            .env("CRABLANGC_BOOTSTRAP", "1");
        if let Some(linker) = builder.linker(self.compiler.host) {
            cmd.env("CRABLANGDOC_LINKER", linker);
        }
        if builder.is_fuse_ld_lld(self.compiler.host) {
            cmd.env(
                "CRABLANGDOC_LLD_NO_THREADS",
                util::lld_flag_no_threads(self.compiler.host.contains("windows")),
            );
        }
        try_run(builder, &mut cmd);
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangdocJSStd {
    pub target: TargetSelection,
}

impl Step for CrabLangdocJSStd {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.suite_path("tests/crablangdoc-js-std")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangdocJSStd { target: run.target });
    }

    fn run(self, builder: &Builder<'_>) {
        if let Some(ref nodejs) = builder.config.nodejs {
            let mut command = Command::new(nodejs);
            command
                .arg(builder.src.join("src/tools/crablangdoc-js/tester.js"))
                .arg("--crate-name")
                .arg("std")
                .arg("--resource-suffix")
                .arg(&builder.version)
                .arg("--doc-folder")
                .arg(builder.doc_out(self.target))
                .arg("--test-folder")
                .arg(builder.src.join("tests/crablangdoc-js-std"));
            for path in &builder.paths {
                if let Some(p) =
                    util::is_valid_test_suite_arg(path, "tests/crablangdoc-js-std", builder)
                {
                    if !p.ends_with(".js") {
                        eprintln!("A non-js file was given: `{}`", path.display());
                        panic!("Cannot run crablangdoc-js-std tests");
                    }
                    command.arg("--test-file").arg(path);
                }
            }
            builder.ensure(crate::doc::Std {
                target: self.target,
                stage: builder.top_stage,
                format: DocumentationFormat::HTML,
            });
            builder.run(&mut command);
        } else {
            builder.info("No nodejs found, skipping \"tests/crablangdoc-js-std\" tests");
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangdocJSNotStd {
    pub target: TargetSelection,
    pub compiler: Compiler,
}

impl Step for CrabLangdocJSNotStd {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.suite_path("tests/crablangdoc-js")
    }

    fn make_run(run: RunConfig<'_>) {
        let compiler = run.builder.compiler(run.builder.top_stage, run.build_triple());
        run.builder.ensure(CrabLangdocJSNotStd { target: run.target, compiler });
    }

    fn run(self, builder: &Builder<'_>) {
        if builder.config.nodejs.is_some() {
            builder.ensure(Compiletest {
                compiler: self.compiler,
                target: self.target,
                mode: "js-doc-test",
                suite: "crablangdoc-js",
                path: "tests/crablangdoc-js",
                compare_mode: None,
            });
        } else {
            builder.info("No nodejs found, skipping \"tests/crablangdoc-js\" tests");
        }
    }
}

fn get_browser_ui_test_version_inner(npm: &Path, global: bool) -> Option<String> {
    let mut command = Command::new(&npm);
    command.arg("list").arg("--parseable").arg("--long").arg("--depth=0");
    if global {
        command.arg("--global");
    }
    let lines = command
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or(String::new());
    lines
        .lines()
        .find_map(|l| l.split(':').nth(1)?.strip_prefix("browser-ui-test@"))
        .map(|v| v.to_owned())
}

fn get_browser_ui_test_version(npm: &Path) -> Option<String> {
    get_browser_ui_test_version_inner(npm, false)
        .or_else(|| get_browser_ui_test_version_inner(npm, true))
}

fn compare_browser_ui_test_version(installed_version: &str, src: &Path) {
    match fs::read_to_string(
        src.join("src/ci/docker/host-x86_64/x86_64-gnu-tools/browser-ui-test.version"),
    ) {
        Ok(v) => {
            if v.trim() != installed_version {
                eprintln!(
                    "⚠️ Installed version of browser-ui-test (`{}`) is different than the \
                     one used in the CI (`{}`)",
                    installed_version, v
                );
                eprintln!(
                    "You can install this version using `npm update browser-ui-test` or by using \
                     `npm install browser-ui-test@{}`",
                    v,
                );
            }
        }
        Err(e) => eprintln!("Couldn't find the CI browser-ui-test version: {:?}", e),
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangdocGUI {
    pub target: TargetSelection,
    pub compiler: Compiler,
}

impl Step for CrabLangdocGUI {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        let run = run.suite_path("tests/crablangdoc-gui");
        run.lazy_default_condition(Box::new(move || {
            builder.config.nodejs.is_some()
                && builder
                    .config
                    .npm
                    .as_ref()
                    .map(|p| get_browser_ui_test_version(p).is_some())
                    .unwrap_or(false)
        }))
    }

    fn make_run(run: RunConfig<'_>) {
        let compiler = run.builder.compiler(run.builder.top_stage, run.build_triple());
        run.builder.ensure(CrabLangdocGUI { target: run.target, compiler });
    }

    fn run(self, builder: &Builder<'_>) {
        let nodejs = builder.config.nodejs.as_ref().expect("nodejs isn't available");
        let npm = builder.config.npm.as_ref().expect("npm isn't available");

        builder.ensure(compile::Std::new(self.compiler, self.target));

        // The goal here is to check if the necessary packages are installed, and if not, we
        // panic.
        match get_browser_ui_test_version(&npm) {
            Some(version) => {
                // We also check the version currently used in CI and emit a warning if it's not the
                // same one.
                compare_browser_ui_test_version(&version, &builder.build.src);
            }
            None => {
                eprintln!(
                    "error: crablangdoc-gui test suite cannot be run because npm `browser-ui-test` \
                     dependency is missing",
                );
                eprintln!(
                    "If you want to install the `{0}` dependency, run `npm install {0}`",
                    "browser-ui-test",
                );
                panic!("Cannot run crablangdoc-gui tests");
            }
        }

        let out_dir = builder.test_out(self.target).join("crablangdoc-gui");

        // We remove existing folder to be sure there won't be artifacts remaining.
        builder.clear_if_dirty(&out_dir, &builder.crablangdoc(self.compiler));

        let src_path = builder.build.src.join("tests/crablangdoc-gui/src");
        // We generate docs for the libraries present in the crablangdoc-gui's src folder.
        for entry in src_path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                let mut cargo = Command::new(&builder.initial_cargo);
                cargo
                    .arg("doc")
                    .arg("--target-dir")
                    .arg(&out_dir)
                    .env("CRABLANGC_BOOTSTRAP", "1")
                    .env("CRABLANGDOC", builder.crablangdoc(self.compiler))
                    .env("CRABLANGC", builder.crablangc(self.compiler))
                    .current_dir(path);
                // FIXME: implement a `// compile-flags` command or similar
                //        instead of hard-coding this test
                if entry.file_name() == "link_to_definition" {
                    cargo.env("CRABLANGDOCFLAGS", "-Zunstable-options --generate-link-to-definition");
                } else if entry.file_name() == "scrape_examples" {
                    cargo.arg("-Zcrablangdoc-scrape-examples");
                }
                builder.run(&mut cargo);
            }
        }

        // We now run GUI tests.
        let mut command = Command::new(&nodejs);
        command
            .arg(builder.build.src.join("src/tools/crablangdoc-gui/tester.js"))
            .arg("--jobs")
            .arg(&builder.jobs().to_string())
            .arg("--doc-folder")
            .arg(out_dir.join("doc"))
            .arg("--tests-folder")
            .arg(builder.build.src.join("tests/crablangdoc-gui"));
        for path in &builder.paths {
            if let Some(p) = util::is_valid_test_suite_arg(path, "tests/crablangdoc-gui", builder) {
                if !p.ends_with(".goml") {
                    eprintln!("A non-goml file was given: `{}`", path.display());
                    panic!("Cannot run crablangdoc-gui tests");
                }
                if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                    command.arg("--file").arg(name);
                }
            }
        }
        for test_arg in builder.config.cmd.test_args() {
            command.arg(test_arg);
        }
        builder.run(&mut command);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Tidy;

impl Step for Tidy {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    /// Runs the `tidy` tool.
    ///
    /// This tool in `src/tools` checks up on various bits and pieces of style and
    /// otherwise just implements a few lint-like checks that are specific to the
    /// compiler itself.
    ///
    /// Once tidy passes, this step also runs `fmt --check` if tests are being run
    /// for the `dev` or `nightly` channels.
    fn run(self, builder: &Builder<'_>) {
        let mut cmd = builder.tool_cmd(Tool::Tidy);
        cmd.arg(&builder.src);
        cmd.arg(&builder.initial_cargo);
        cmd.arg(&builder.out);
        // Tidy is heavily IO constrained. Still respect `-j`, but use a higher limit if `jobs` hasn't been configured.
        let jobs = builder.config.jobs.unwrap_or_else(|| {
            8 * std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get) as u32
        });
        cmd.arg(jobs.to_string());
        if builder.is_verbose() {
            cmd.arg("--verbose");
        }
        if builder.config.cmd.bless() {
            cmd.arg("--bless");
        }

        if builder.config.channel == "dev" || builder.config.channel == "nightly" {
            builder.info("fmt check");
            if builder.initial_crablangfmt().is_none() {
                let inferred_crablangfmt_dir = builder.initial_crablangc.parent().unwrap();
                eprintln!(
                    "\
error: no `crablangfmt` binary found in {PATH}
info: `crablang.channel` is currently set to \"{CHAN}\"
help: if you are testing a beta branch, set `crablang.channel` to \"beta\" in the `config.toml` file
help: to skip test's attempt to check tidiness, pass `--exclude src/tools/tidy` to `x.py test`",
                    PATH = inferred_crablangfmt_dir.display(),
                    CHAN = builder.config.channel,
                );
                crate::detail_exit(1);
            }
            crate::format::format(&builder, !builder.config.cmd.bless(), &[]);
        }

        builder.info("tidy check");
        try_run(builder, &mut cmd);

        builder.ensure(ExpandYamlAnchors {});
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/tidy")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Tidy);
    }
}

/// Runs tidy's own tests.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TidySelfTest;

impl Step for TidySelfTest {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("tidyselftest")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(TidySelfTest);
    }

    fn run(self, builder: &Builder<'_>) {
        let bootstrap_host = builder.config.build;
        let compiler = builder.compiler(0, bootstrap_host);
        let cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolBootstrap,
            bootstrap_host,
            "test",
            "src/tools/tidy",
            SourceType::InTree,
            &[],
        );
        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExpandYamlAnchors;

impl Step for ExpandYamlAnchors {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    /// Ensure the `generate-ci-config` tool was run locally.
    ///
    /// The tool in `src/tools` reads the CI definition in `src/ci/builders.yml` and generates the
    /// appropriate configuration for all our CI providers. This step ensures the tool was called
    /// by the user before committing CI changes.
    fn run(self, builder: &Builder<'_>) {
        builder.info("Ensuring the YAML anchors in the GitHub Actions config were expanded");
        try_run(
            builder,
            &mut builder.tool_cmd(Tool::ExpandYamlAnchors).arg("check").arg(&builder.src),
        );
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/expand-yaml-anchors")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(ExpandYamlAnchors);
    }
}

fn testdir(builder: &Builder<'_>, host: TargetSelection) -> PathBuf {
    builder.out.join(host.triple).join("test")
}

macro_rules! default_test {
    ($name:ident { path: $path:expr, mode: $mode:expr, suite: $suite:expr }) => {
        test!($name { path: $path, mode: $mode, suite: $suite, default: true, host: false });
    };
}

macro_rules! default_test_with_compare_mode {
    ($name:ident { path: $path:expr, mode: $mode:expr, suite: $suite:expr,
                   compare_mode: $compare_mode:expr }) => {
        test_with_compare_mode!($name {
            path: $path,
            mode: $mode,
            suite: $suite,
            default: true,
            host: false,
            compare_mode: $compare_mode
        });
    };
}

macro_rules! host_test {
    ($name:ident { path: $path:expr, mode: $mode:expr, suite: $suite:expr }) => {
        test!($name { path: $path, mode: $mode, suite: $suite, default: true, host: true });
    };
}

macro_rules! test {
    ($name:ident { path: $path:expr, mode: $mode:expr, suite: $suite:expr, default: $default:expr,
                   host: $host:expr }) => {
        test_definitions!($name {
            path: $path,
            mode: $mode,
            suite: $suite,
            default: $default,
            host: $host,
            compare_mode: None
        });
    };
}

macro_rules! test_with_compare_mode {
    ($name:ident { path: $path:expr, mode: $mode:expr, suite: $suite:expr, default: $default:expr,
                   host: $host:expr, compare_mode: $compare_mode:expr }) => {
        test_definitions!($name {
            path: $path,
            mode: $mode,
            suite: $suite,
            default: $default,
            host: $host,
            compare_mode: Some($compare_mode)
        });
    };
}

macro_rules! test_definitions {
    ($name:ident {
        path: $path:expr,
        mode: $mode:expr,
        suite: $suite:expr,
        default: $default:expr,
        host: $host:expr,
        compare_mode: $compare_mode:expr
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            pub compiler: Compiler,
            pub target: TargetSelection,
        }

        impl Step for $name {
            type Output = ();
            const DEFAULT: bool = $default;
            const ONLY_HOSTS: bool = $host;

            fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
                run.suite_path($path)
            }

            fn make_run(run: RunConfig<'_>) {
                let compiler = run.builder.compiler(run.builder.top_stage, run.build_triple());

                run.builder.ensure($name { compiler, target: run.target });
            }

            fn run(self, builder: &Builder<'_>) {
                builder.ensure(Compiletest {
                    compiler: self.compiler,
                    target: self.target,
                    mode: $mode,
                    suite: $suite,
                    path: $path,
                    compare_mode: $compare_mode,
                })
            }
        }
    };
}

default_test!(Ui { path: "tests/ui", mode: "ui", suite: "ui" });

default_test!(RunPassValgrind {
    path: "tests/run-pass-valgrind",
    mode: "run-pass-valgrind",
    suite: "run-pass-valgrind"
});

default_test!(MirOpt { path: "tests/mir-opt", mode: "mir-opt", suite: "mir-opt" });

default_test!(Codegen { path: "tests/codegen", mode: "codegen", suite: "codegen" });

default_test!(CodegenUnits {
    path: "tests/codegen-units",
    mode: "codegen-units",
    suite: "codegen-units"
});

default_test!(Incremental { path: "tests/incremental", mode: "incremental", suite: "incremental" });

default_test_with_compare_mode!(Debuginfo {
    path: "tests/debuginfo",
    mode: "debuginfo",
    suite: "debuginfo",
    compare_mode: "split-dwarf"
});

host_test!(UiFullDeps { path: "tests/ui-fulldeps", mode: "ui", suite: "ui-fulldeps" });

host_test!(CrabLangdoc { path: "tests/crablangdoc", mode: "crablangdoc", suite: "crablangdoc" });
host_test!(CrabLangdocUi { path: "tests/crablangdoc-ui", mode: "ui", suite: "crablangdoc-ui" });

host_test!(CrabLangdocJson { path: "tests/crablangdoc-json", mode: "crablangdoc-json", suite: "crablangdoc-json" });

host_test!(Pretty { path: "tests/pretty", mode: "pretty", suite: "pretty" });

default_test!(RunMake { path: "tests/run-make", mode: "run-make", suite: "run-make" });

host_test!(RunMakeFullDeps {
    path: "tests/run-make-fulldeps",
    mode: "run-make",
    suite: "run-make-fulldeps"
});

default_test!(Assembly { path: "tests/assembly", mode: "assembly", suite: "assembly" });

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Compiletest {
    compiler: Compiler,
    target: TargetSelection,
    mode: &'static str,
    suite: &'static str,
    path: &'static str,
    compare_mode: Option<&'static str>,
}

impl Step for Compiletest {
    type Output = ();

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    /// Executes the `compiletest` tool to run a suite of tests.
    ///
    /// Compiles all tests with `compiler` for `target` with the specified
    /// compiletest `mode` and `suite` arguments. For example `mode` can be
    /// "run-pass" or `suite` can be something like `debuginfo`.
    fn run(self, builder: &Builder<'_>) {
        if builder.top_stage == 0 && env::var("COMPILETEST_FORCE_STAGE0").is_err() {
            eprintln!("\
error: `--stage 0` runs compiletest on the beta compiler, not your local changes, and will almost always cause tests to fail
help: to test the compiler, use `--stage 1` instead
help: to test the standard library, use `--stage 0 library/std` instead
note: if you're sure you want to do this, please open an issue as to why. In the meantime, you can override this with `COMPILETEST_FORCE_STAGE0=1`."
            );
            crate::detail_exit(1);
        }

        let compiler = self.compiler;
        let target = self.target;
        let mode = self.mode;
        let suite = self.suite;

        // Path for test suite
        let suite_path = self.path;

        // Skip codegen tests if they aren't enabled in configuration.
        if !builder.config.codegen_tests && suite == "codegen" {
            return;
        }

        if suite == "debuginfo" {
            builder
                .ensure(dist::DebuggerScripts { sysroot: builder.sysroot(compiler), host: target });
        }

        if suite.ends_with("fulldeps") {
            builder.ensure(compile::CrabLangc::new(compiler, target));
        }

        builder.ensure(compile::Std::new(compiler, target));
        // ensure that `libproc_macro` is available on the host.
        builder.ensure(compile::Std::new(compiler, compiler.host));

        // Also provide `crablang_test_helpers` for the host.
        builder.ensure(TestHelpers { target: compiler.host });

        // As well as the target, except for plain wasm32, which can't build it
        if !target.contains("wasm") || target.contains("emscripten") {
            builder.ensure(TestHelpers { target });
        }

        builder.ensure(RemoteCopyLibs { compiler, target });

        let mut cmd = builder.tool_cmd(Tool::Compiletest);

        // compiletest currently has... a lot of arguments, so let's just pass all
        // of them!

        cmd.arg("--compile-lib-path").arg(builder.crablangc_libdir(compiler));
        cmd.arg("--run-lib-path").arg(builder.sysroot_libdir(compiler, target));
        cmd.arg("--crablangc-path").arg(builder.crablangc(compiler));

        let is_crablangdoc = suite.ends_with("crablangdoc-ui") || suite.ends_with("crablangdoc-js");

        // Avoid depending on crablangdoc when we don't need it.
        if mode == "crablangdoc"
            || mode == "run-make"
            || (mode == "ui" && is_crablangdoc)
            || mode == "js-doc-test"
            || mode == "crablangdoc-json"
        {
            cmd.arg("--crablangdoc-path").arg(builder.crablangdoc(compiler));
        }

        if mode == "crablangdoc-json" {
            // Use the beta compiler for jsondocck
            let json_compiler = compiler.with_stage(0);
            cmd.arg("--jsondocck-path")
                .arg(builder.ensure(tool::JsonDocCk { compiler: json_compiler, target }));
            cmd.arg("--jsondoclint-path")
                .arg(builder.ensure(tool::JsonDocLint { compiler: json_compiler, target }));
        }

        if mode == "run-make" {
            let crablang_demangler = builder
                .ensure(tool::CrabLangDemangler {
                    compiler,
                    target: compiler.host,
                    extra_features: Vec::new(),
                })
                .expect("in-tree tool");
            cmd.arg("--crablang-demangler-path").arg(crablang_demangler);
        }

        cmd.arg("--src-base").arg(builder.src.join("tests").join(suite));
        cmd.arg("--build-base").arg(testdir(builder, compiler.host).join(suite));
        cmd.arg("--sysroot-base").arg(builder.sysroot(compiler));
        cmd.arg("--stage-id").arg(format!("stage{}-{}", compiler.stage, target));
        cmd.arg("--suite").arg(suite);
        cmd.arg("--mode").arg(mode);
        cmd.arg("--target").arg(target.crablangc_target_arg());
        cmd.arg("--host").arg(&*compiler.host.triple);
        cmd.arg("--llvm-filecheck").arg(builder.llvm_filecheck(builder.config.build));

        if builder.config.cmd.bless() {
            cmd.arg("--bless");
        }

        if builder.config.cmd.force_rerun() {
            cmd.arg("--force-rerun");
        }

        let compare_mode =
            builder.config.cmd.compare_mode().or_else(|| {
                if builder.config.test_compare_mode { self.compare_mode } else { None }
            });

        if let Some(ref pass) = builder.config.cmd.pass() {
            cmd.arg("--pass");
            cmd.arg(pass);
        }

        if let Some(ref run) = builder.config.cmd.run() {
            cmd.arg("--run");
            cmd.arg(run);
        }

        if let Some(ref nodejs) = builder.config.nodejs {
            cmd.arg("--nodejs").arg(nodejs);
        }
        if let Some(ref npm) = builder.config.npm {
            cmd.arg("--npm").arg(npm);
        }
        if builder.config.crablang_optimize_tests {
            cmd.arg("--optimize-tests");
        }
        if builder.config.cmd.only_modified() {
            cmd.arg("--only-modified");
        }

        let mut flags = if is_crablangdoc { Vec::new() } else { vec!["-Crpath".to_string()] };
        flags.push(format!("-Cdebuginfo={}", builder.config.crablang_debuginfo_level_tests));
        flags.extend(builder.config.cmd.crablangc_args().iter().map(|s| s.to_string()));

        if let Some(linker) = builder.linker(target) {
            cmd.arg("--linker").arg(linker);
        }

        let mut hostflags = flags.clone();
        hostflags.push(format!("-Lnative={}", builder.test_helpers_out(compiler.host).display()));
        hostflags.extend(builder.lld_flags(compiler.host));
        for flag in hostflags {
            cmd.arg("--host-crablangcflags").arg(flag);
        }

        let mut targetflags = flags;
        targetflags.push(format!("-Lnative={}", builder.test_helpers_out(target).display()));
        targetflags.extend(builder.lld_flags(target));
        for flag in targetflags {
            cmd.arg("--target-crablangcflags").arg(flag);
        }

        cmd.arg("--python").arg(builder.python());

        if let Some(ref gdb) = builder.config.gdb {
            cmd.arg("--gdb").arg(gdb);
        }

        let run = |cmd: &mut Command| {
            cmd.output().map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or_else(|| panic!("{:?} failed {:?}", cmd, output))
                    .to_string()
            })
        };
        let lldb_exe = "lldb";
        let lldb_version = Command::new(lldb_exe)
            .arg("--version")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .ok();
        if let Some(ref vers) = lldb_version {
            cmd.arg("--lldb-version").arg(vers);
            let lldb_python_dir = run(Command::new(lldb_exe).arg("-P")).ok();
            if let Some(ref dir) = lldb_python_dir {
                cmd.arg("--lldb-python-dir").arg(dir);
            }
        }

        if util::forcing_clang_based_tests() {
            let clang_exe = builder.llvm_out(target).join("bin").join("clang");
            cmd.arg("--run-clang-based-tests-with").arg(clang_exe);
        }

        for exclude in &builder.config.exclude {
            cmd.arg("--skip");
            cmd.arg(&exclude.path);
        }

        // Get paths from cmd args
        let paths = match &builder.config.cmd {
            Subcommand::Test { ref paths, .. } => &paths[..],
            _ => &[],
        };

        // Get test-args by striping suite path
        let mut test_args: Vec<&str> = paths
            .iter()
            .filter_map(|p| util::is_valid_test_suite_arg(p, suite_path, builder))
            .collect();

        test_args.append(&mut builder.config.cmd.test_args());
        test_args.extend(builder.config.free_args.iter().map(|s| s.as_str()));

        // On Windows, replace forward slashes in test-args by backslashes
        // so the correct filters are passed to libtest
        if cfg!(windows) {
            let test_args_win: Vec<String> =
                test_args.iter().map(|s| s.replace("/", "\\")).collect();
            cmd.args(&test_args_win);
        } else {
            cmd.args(&test_args);
        }

        if builder.is_verbose() {
            cmd.arg("--verbose");
        }

        cmd.arg("--json");

        let mut llvm_components_passed = false;
        let mut copts_passed = false;
        if builder.config.llvm_enabled() {
            let llvm::LlvmResult { llvm_config, .. } =
                builder.ensure(llvm::Llvm { target: builder.config.build });
            if !builder.config.dry_run() {
                let llvm_version = output(Command::new(&llvm_config).arg("--version"));
                let llvm_components = output(Command::new(&llvm_config).arg("--components"));
                // Remove trailing newline from llvm-config output.
                cmd.arg("--llvm-version")
                    .arg(llvm_version.trim())
                    .arg("--llvm-components")
                    .arg(llvm_components.trim());
                llvm_components_passed = true;
            }
            if !builder.is_crablang_llvm(target) {
                cmd.arg("--system-llvm");
            }

            // Tests that use compiler libraries may inherit the `-lLLVM` link
            // requirement, but the `-L` library path is not propagated across
            // separate compilations. We can add LLVM's library path to the
            // platform-specific environment variable as a workaround.
            if !builder.config.dry_run() && suite.ends_with("fulldeps") {
                let llvm_libdir = output(Command::new(&llvm_config).arg("--libdir"));
                add_link_lib_path(vec![llvm_libdir.trim().into()], &mut cmd);
            }

            // Only pass correct values for these flags for the `run-make` suite as it
            // requires that a C++ compiler was configured which isn't always the case.
            if !builder.config.dry_run() && matches!(suite, "run-make" | "run-make-fulldeps") {
                // The llvm/bin directory contains many useful cross-platform
                // tools. Pass the path to run-make tests so they can use them.
                let llvm_bin_path = llvm_config
                    .parent()
                    .expect("Expected llvm-config to be contained in directory");
                assert!(llvm_bin_path.is_dir());
                cmd.arg("--llvm-bin-dir").arg(llvm_bin_path);

                // If LLD is available, add it to the PATH
                if builder.config.lld_enabled {
                    let lld_install_root =
                        builder.ensure(llvm::Lld { target: builder.config.build });

                    let lld_bin_path = lld_install_root.join("bin");

                    let old_path = env::var_os("PATH").unwrap_or_default();
                    let new_path = env::join_paths(
                        std::iter::once(lld_bin_path).chain(env::split_paths(&old_path)),
                    )
                    .expect("Could not add LLD bin path to PATH");
                    cmd.env("PATH", new_path);
                }
            }
        }

        // Only pass correct values for these flags for the `run-make` suite as it
        // requires that a C++ compiler was configured which isn't always the case.
        if !builder.config.dry_run() && matches!(suite, "run-make" | "run-make-fulldeps") {
            cmd.arg("--cc")
                .arg(builder.cc(target))
                .arg("--cxx")
                .arg(builder.cxx(target).unwrap())
                .arg("--cflags")
                .arg(builder.cflags(target, GitRepo::CrabLangc, CLang::C).join(" "))
                .arg("--cxxflags")
                .arg(builder.cflags(target, GitRepo::CrabLangc, CLang::Cxx).join(" "));
            copts_passed = true;
            if let Some(ar) = builder.ar(target) {
                cmd.arg("--ar").arg(ar);
            }
        }

        if !llvm_components_passed {
            cmd.arg("--llvm-components").arg("");
        }
        if !copts_passed {
            cmd.arg("--cc")
                .arg("")
                .arg("--cxx")
                .arg("")
                .arg("--cflags")
                .arg("")
                .arg("--cxxflags")
                .arg("");
        }

        if builder.remote_tested(target) {
            cmd.arg("--remote-test-client").arg(builder.tool_exe(Tool::RemoteTestClient));
        }

        // Running a C compiler on MSVC requires a few env vars to be set, to be
        // sure to set them here.
        //
        // Note that if we encounter `PATH` we make sure to append to our own `PATH`
        // rather than stomp over it.
        if target.contains("msvc") {
            for &(ref k, ref v) in builder.cc[&target].env() {
                if k != "PATH" {
                    cmd.env(k, v);
                }
            }
        }
        cmd.env("CRABLANGC_BOOTSTRAP", "1");
        // Override the crablangc version used in symbol hashes to reduce the amount of normalization
        // needed when diffing test output.
        cmd.env("CRABLANGC_FORCE_CRABLANGC_VERSION", "compiletest");
        cmd.env("DOC_CRABLANG_LANG_ORG_CHANNEL", builder.doc_crablang_lang_org_channel());
        builder.add_crablang_test_threads(&mut cmd);

        if builder.config.sanitizers_enabled(target) {
            cmd.env("CRABLANGC_SANITIZER_SUPPORT", "1");
        }

        if builder.config.profiler_enabled(target) {
            cmd.env("CRABLANGC_PROFILER_SUPPORT", "1");
        }

        cmd.env("CRABLANG_TEST_TMPDIR", builder.tempdir());

        cmd.arg("--adb-path").arg("adb");
        cmd.arg("--adb-test-dir").arg(ADB_TEST_DIR);
        if target.contains("android") {
            // Assume that cc for this target comes from the android sysroot
            cmd.arg("--android-cross-path")
                .arg(builder.cc(target).parent().unwrap().parent().unwrap());
        } else {
            cmd.arg("--android-cross-path").arg("");
        }

        if builder.config.cmd.crablangfix_coverage() {
            cmd.arg("--crablangfix-coverage");
        }

        cmd.env("BOOTSTRAP_CARGO", &builder.initial_cargo);

        cmd.arg("--channel").arg(&builder.config.channel);

        if let Some(commit) = builder.config.download_crablangc_commit() {
            cmd.env("FAKE_DOWNLOAD_CRABLANGC_PREFIX", format!("/crablangc/{commit}"));
        }

        builder.ci_env.force_coloring_in_ci(&mut cmd);

        builder.info(&format!(
            "Check compiletest suite={} mode={} ({} -> {})",
            suite, mode, &compiler.host, target
        ));
        let _time = util::timeit(&builder);
        crate::render_tests::try_run_tests(builder, &mut cmd);

        if let Some(compare_mode) = compare_mode {
            cmd.arg("--compare-mode").arg(compare_mode);
            builder.info(&format!(
                "Check compiletest suite={} mode={} compare_mode={} ({} -> {})",
                suite, mode, compare_mode, &compiler.host, target
            ));
            let _time = util::timeit(&builder);
            crate::render_tests::try_run_tests(builder, &mut cmd);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BookTest {
    compiler: Compiler,
    path: PathBuf,
    name: &'static str,
    is_ext_doc: bool,
}

impl Step for BookTest {
    type Output = ();
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    /// Runs the documentation tests for a book in `src/doc`.
    ///
    /// This uses the `crablangdoc` that sits next to `compiler`.
    fn run(self, builder: &Builder<'_>) {
        // External docs are different from local because:
        // - Some books need pre-processing by mdbook before being tested.
        // - They need to save their state to toolstate.
        // - They are only tested on the "checktools" builders.
        //
        // The local docs are tested by default, and we don't want to pay the
        // cost of building mdbook, so they use `crablangdoc --test` directly.
        // Also, the unstable book is special because SUMMARY.md is generated,
        // so it is easier to just run `crablangdoc` on its files.
        if self.is_ext_doc {
            self.run_ext_doc(builder);
        } else {
            self.run_local_doc(builder);
        }
    }
}

impl BookTest {
    /// This runs the equivalent of `mdbook test` (via the crablangbook wrapper)
    /// which in turn runs `crablangdoc --test` on each file in the book.
    fn run_ext_doc(self, builder: &Builder<'_>) {
        let compiler = self.compiler;

        builder.ensure(compile::Std::new(compiler, compiler.host));

        // mdbook just executes a binary named "crablangdoc", so we need to update
        // PATH so that it points to our crablangdoc.
        let mut crablangdoc_path = builder.crablangdoc(compiler);
        crablangdoc_path.pop();
        let old_path = env::var_os("PATH").unwrap_or_default();
        let new_path = env::join_paths(iter::once(crablangdoc_path).chain(env::split_paths(&old_path)))
            .expect("could not add crablangdoc to PATH");

        let mut crablangbook_cmd = builder.tool_cmd(Tool::CrabLangbook);
        let path = builder.src.join(&self.path);
        // Books often have feature-gated example text.
        crablangbook_cmd.env("CRABLANGC_BOOTSTRAP", "1");
        crablangbook_cmd.env("PATH", new_path).arg("test").arg(path);
        builder.add_crablang_test_threads(&mut crablangbook_cmd);
        builder.info(&format!("Testing crablangbook {}", self.path.display()));
        let _time = util::timeit(&builder);
        let toolstate = if try_run(builder, &mut crablangbook_cmd) {
            ToolState::TestPass
        } else {
            ToolState::TestFail
        };
        builder.save_toolstate(self.name, toolstate);
    }

    /// This runs `crablangdoc --test` on all `.md` files in the path.
    fn run_local_doc(self, builder: &Builder<'_>) {
        let compiler = self.compiler;

        builder.ensure(compile::Std::new(compiler, compiler.host));

        // Do a breadth-first traversal of the `src/doc` directory and just run
        // tests for all files that end in `*.md`
        let mut stack = vec![builder.src.join(self.path)];
        let _time = util::timeit(&builder);
        let mut files = Vec::new();
        while let Some(p) = stack.pop() {
            if p.is_dir() {
                stack.extend(t!(p.read_dir()).map(|p| t!(p).path()));
                continue;
            }

            if p.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            files.push(p);
        }

        files.sort();

        for file in files {
            markdown_test(builder, compiler, &file);
        }
    }
}

macro_rules! test_book {
    ($($name:ident, $path:expr, $book_name:expr, default=$default:expr;)+) => {
        $(
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            pub struct $name {
                compiler: Compiler,
            }

            impl Step for $name {
                type Output = ();
                const DEFAULT: bool = $default;
                const ONLY_HOSTS: bool = true;

                fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
                    run.path($path)
                }

                fn make_run(run: RunConfig<'_>) {
                    run.builder.ensure($name {
                        compiler: run.builder.compiler(run.builder.top_stage, run.target),
                    });
                }

                fn run(self, builder: &Builder<'_>) {
                    builder.ensure(BookTest {
                        compiler: self.compiler,
                        path: PathBuf::from($path),
                        name: $book_name,
                        is_ext_doc: !$default,
                    });
                }
            }
        )+
    }
}

test_book!(
    Nomicon, "src/doc/nomicon", "nomicon", default=false;
    Reference, "src/doc/reference", "reference", default=false;
    CrabLangdocBook, "src/doc/crablangdoc", "crablangdoc", default=true;
    CrabLangcBook, "src/doc/crablangc", "crablangc", default=true;
    CrabLangByExample, "src/doc/crablang-by-example", "crablang-by-example", default=false;
    EmbeddedBook, "src/doc/embedded-book", "embedded-book", default=false;
    TheBook, "src/doc/book", "book", default=false;
    UnstableBook, "src/doc/unstable-book", "unstable-book", default=true;
    EditionGuide, "src/doc/edition-guide", "edition-guide", default=false;
);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ErrorIndex {
    compiler: Compiler,
}

impl Step for ErrorIndex {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/error_index_generator")
    }

    fn make_run(run: RunConfig<'_>) {
        // error_index_generator depends on libcrablangdoc. Use the compiler that
        // is normally used to build crablangdoc for other tests (like compiletest
        // tests in tests/crablangdoc) so that it shares the same artifacts.
        let compiler = run.builder.compiler(run.builder.top_stage, run.builder.config.build);
        run.builder.ensure(ErrorIndex { compiler });
    }

    /// Runs the error index generator tool to execute the tests located in the error
    /// index.
    ///
    /// The `error_index_generator` tool lives in `src/tools` and is used to
    /// generate a markdown file from the error indexes of the code base which is
    /// then passed to `crablangdoc --test`.
    fn run(self, builder: &Builder<'_>) {
        let compiler = self.compiler;

        let dir = testdir(builder, compiler.host);
        t!(fs::create_dir_all(&dir));
        let output = dir.join("error-index.md");

        let mut tool = tool::ErrorIndex::command(builder);
        tool.arg("markdown").arg(&output);

        builder.info(&format!("Testing error-index stage{}", compiler.stage));
        let _time = util::timeit(&builder);
        builder.run_quiet(&mut tool);
        // The tests themselves need to link to std, so make sure it is
        // available.
        builder.ensure(compile::Std::new(compiler, compiler.host));
        markdown_test(builder, compiler, &output);
    }
}

fn markdown_test(builder: &Builder<'_>, compiler: Compiler, markdown: &Path) -> bool {
    if let Ok(contents) = fs::read_to_string(markdown) {
        if !contents.contains("```") {
            return true;
        }
    }

    builder.info(&format!("doc tests for: {}", markdown.display()));
    let mut cmd = builder.crablangdoc_cmd(compiler);
    builder.add_crablang_test_threads(&mut cmd);
    // allow for unstable options such as new editions
    cmd.arg("-Z");
    cmd.arg("unstable-options");
    cmd.arg("--test");
    cmd.arg(markdown);
    cmd.env("CRABLANGC_BOOTSTRAP", "1");

    let test_args = builder.config.cmd.test_args().join(" ");
    cmd.arg("--test-args").arg(test_args);

    if builder.config.verbose_tests {
        try_run(builder, &mut cmd)
    } else {
        try_run_quiet(builder, &mut cmd)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangcGuide;

impl Step for CrabLangcGuide {
    type Output = ();
    const DEFAULT: bool = false;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/doc/crablangc-dev-guide")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangcGuide);
    }

    fn run(self, builder: &Builder<'_>) {
        let relative_path = Path::new("src").join("doc").join("crablangc-dev-guide");
        builder.update_submodule(&relative_path);

        let src = builder.src.join(relative_path);
        let mut crablangbook_cmd = builder.tool_cmd(Tool::CrabLangbook);
        let toolstate = if try_run(builder, crablangbook_cmd.arg("linkcheck").arg(&src)) {
            ToolState::TestPass
        } else {
            ToolState::TestFail
        };
        builder.save_toolstate("crablangc-dev-guide", toolstate);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateLibcrablangc {
    compiler: Compiler,
    target: TargetSelection,
    test_kind: TestKind,
    crates: Vec<Interned<String>>,
}

impl Step for CrateLibcrablangc {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.crate_or_deps("crablangc-main")
    }

    fn make_run(run: RunConfig<'_>) {
        let builder = run.builder;
        let host = run.build_triple();
        let compiler = builder.compiler_for(builder.top_stage, host, host);
        let crates = run
            .paths
            .iter()
            .map(|p| builder.crate_paths[&p.assert_single_path().path].clone())
            .collect();
        let test_kind = builder.kind.into();

        builder.ensure(CrateLibcrablangc { compiler, target: run.target, test_kind, crates });
    }

    fn run(self, builder: &Builder<'_>) {
        builder.ensure(Crate {
            compiler: self.compiler,
            target: self.target,
            mode: Mode::CrabLangc,
            test_kind: self.test_kind,
            crates: self.crates,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Crate {
    pub compiler: Compiler,
    pub target: TargetSelection,
    pub mode: Mode,
    pub test_kind: TestKind,
    pub crates: Vec<Interned<String>>,
}

impl Step for Crate {
    type Output = ();
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.crate_or_deps("test")
    }

    fn make_run(run: RunConfig<'_>) {
        let builder = run.builder;
        let host = run.build_triple();
        let compiler = builder.compiler_for(builder.top_stage, host, host);
        let test_kind = builder.kind.into();
        let crates = run
            .paths
            .iter()
            .map(|p| builder.crate_paths[&p.assert_single_path().path].clone())
            .collect();

        builder.ensure(Crate { compiler, target: run.target, mode: Mode::Std, test_kind, crates });
    }

    /// Runs all unit tests plus documentation tests for a given crate defined
    /// by a `Cargo.toml` (single manifest)
    ///
    /// This is what runs tests for crates like the standard library, compiler, etc.
    /// It essentially is the driver for running `cargo test`.
    ///
    /// Currently this runs all tests for a DAG by passing a bunch of `-p foo`
    /// arguments, and those arguments are discovered from `cargo metadata`.
    fn run(self, builder: &Builder<'_>) {
        let compiler = self.compiler;
        let target = self.target;
        let mode = self.mode;
        let test_kind = self.test_kind;

        builder.ensure(compile::Std::new(compiler, target));
        builder.ensure(RemoteCopyLibs { compiler, target });

        // If we're not doing a full bootstrap but we're testing a stage2
        // version of libstd, then what we're actually testing is the libstd
        // produced in stage1. Reflect that here by updating the compiler that
        // we're working with automatically.
        let compiler = builder.compiler_for(compiler.stage, compiler.host, target);

        let mut cargo =
            builder.cargo(compiler, mode, SourceType::InTree, target, test_kind.subcommand());
        match mode {
            Mode::Std => {
                compile::std_cargo(builder, target, compiler.stage, &mut cargo);
            }
            Mode::CrabLangc => {
                compile::crablangc_cargo(builder, &mut cargo, target);
            }
            _ => panic!("can only test libraries"),
        };

        // Build up the base `cargo test` command.
        //
        // Pass in some standard flags then iterate over the graph we've discovered
        // in `cargo metadata` with the maps above and figure out what `-p`
        // arguments need to get passed.
        if test_kind.subcommand() == "test" && !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }
        match builder.doc_tests {
            DocTests::Only => {
                cargo.arg("--doc");
            }
            DocTests::No => {
                cargo.args(&["--lib", "--bins", "--examples", "--tests", "--benches"]);
            }
            DocTests::Yes => {}
        }

        for krate in &self.crates {
            cargo.arg("-p").arg(krate);
        }

        // The tests are going to run with the *target* libraries, so we need to
        // ensure that those libraries show up in the LD_LIBRARY_PATH equivalent.
        //
        // Note that to run the compiler we need to run with the *host* libraries,
        // but our wrapper scripts arrange for that to be the case anyway.
        let mut dylib_path = dylib_path();
        dylib_path.insert(0, PathBuf::from(&*builder.sysroot_libdir(compiler, target)));
        cargo.env(dylib_path_var(), env::join_paths(&dylib_path).unwrap());

        cargo.arg("--");
        cargo.args(&builder.config.cmd.test_args());

        cargo.arg("-Z").arg("unstable-options");
        cargo.arg("--format").arg("json");

        if target.contains("emscripten") {
            cargo.env(
                format!("CARGO_TARGET_{}_RUNNER", envify(&target.triple)),
                builder.config.nodejs.as_ref().expect("nodejs not configured"),
            );
        } else if target.starts_with("wasm32") {
            let node = builder.config.nodejs.as_ref().expect("nodejs not configured");
            let runner =
                format!("{} {}/src/etc/wasm32-shim.js", node.display(), builder.src.display());
            cargo.env(format!("CARGO_TARGET_{}_RUNNER", envify(&target.triple)), &runner);
        } else if builder.remote_tested(target) {
            cargo.env(
                format!("CARGO_TARGET_{}_RUNNER", envify(&target.triple)),
                format!("{} run 0", builder.tool_exe(Tool::RemoteTestClient).display()),
            );
        }

        builder.info(&format!(
            "{}{} stage{} ({} -> {})",
            test_kind,
            crate_description(&self.crates),
            compiler.stage,
            &compiler.host,
            target
        ));
        let _time = util::timeit(&builder);
        crate::render_tests::try_run_tests(builder, &mut cargo.into());
    }
}

/// CrabLangdoc is special in various ways, which is why this step is different from `Crate`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrateCrabLangdoc {
    host: TargetSelection,
    test_kind: TestKind,
}

impl Step for CrateCrabLangdoc {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.paths(&["src/libcrablangdoc", "src/tools/crablangdoc"])
    }

    fn make_run(run: RunConfig<'_>) {
        let builder = run.builder;

        let test_kind = builder.kind.into();

        builder.ensure(CrateCrabLangdoc { host: run.target, test_kind });
    }

    fn run(self, builder: &Builder<'_>) {
        let test_kind = self.test_kind;
        let target = self.host;

        let compiler = if builder.download_crablangc() {
            builder.compiler(builder.top_stage, target)
        } else {
            // Use the previous stage compiler to reuse the artifacts that are
            // created when running compiletest for tests/crablangdoc. If this used
            // `compiler`, then it would cause crablangdoc to be built *again*, which
            // isn't really necessary.
            builder.compiler_for(builder.top_stage, target, target)
        };
        builder.ensure(compile::CrabLangc::new(compiler, target));

        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            target,
            test_kind.subcommand(),
            "src/tools/crablangdoc",
            SourceType::InTree,
            &[],
        );
        if test_kind.subcommand() == "test" && !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }
        match builder.doc_tests {
            DocTests::Only => {
                cargo.arg("--doc");
            }
            DocTests::No => {
                cargo.args(&["--lib", "--bins", "--examples", "--tests", "--benches"]);
            }
            DocTests::Yes => {}
        }

        cargo.arg("-p").arg("crablangdoc:0.0.0");

        cargo.arg("--");
        cargo.args(&builder.config.cmd.test_args());

        if self.host.contains("musl") {
            cargo.arg("'-Ctarget-feature=-crt-static'");
        }

        // This is needed for running doctests on libcrablangdoc. This is a bit of
        // an unfortunate interaction with how bootstrap works and how cargo
        // sets up the dylib path, and the fact that the doctest (in
        // html/markdown.rs) links to crablangc-private libs. For stage1, the
        // compiler host dylibs (in stage1/lib) are not the same as the target
        // dylibs (in stage1/lib/crablanglib/...). This is different from a normal
        // crablang distribution where they are the same.
        //
        // On the cargo side, normal tests use `target_process` which handles
        // setting up the dylib for a *target* (stage1/lib/crablanglib/... in this
        // case). However, for doctests it uses `crablangdoc_process` which only
        // sets up the dylib path for the *host* (stage1/lib), which is the
        // wrong directory.
        //
        // Recall that we special-cased `compiler_for(top_stage)` above, so we always use stage1.
        //
        // It should be considered to just stop running doctests on
        // libcrablangdoc. There is only one test, and it doesn't look too
        // important. There might be other ways to avoid this, but it seems
        // pretty convoluted.
        //
        // See also https://github.com/crablang/crablang/issues/13983 where the
        // host vs target dylibs for crablangdoc are consistently tricky to deal
        // with.
        //
        // Note that this set the host libdir for `download_crablangc`, which uses a normal crablang distribution.
        let libdir = if builder.download_crablangc() {
            builder.crablangc_libdir(compiler)
        } else {
            builder.sysroot_libdir(compiler, target).to_path_buf()
        };
        let mut dylib_path = dylib_path();
        dylib_path.insert(0, PathBuf::from(&*libdir));
        cargo.env(dylib_path_var(), env::join_paths(&dylib_path).unwrap());

        if !builder.config.verbose_tests {
            cargo.arg("--quiet");
        }

        builder.info(&format!(
            "{} crablangdoc stage{} ({} -> {})",
            test_kind, compiler.stage, &compiler.host, target
        ));
        let _time = util::timeit(&builder);

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrateCrabLangdocJsonTypes {
    host: TargetSelection,
    test_kind: TestKind,
}

impl Step for CrateCrabLangdocJsonTypes {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/crablangdoc-json-types")
    }

    fn make_run(run: RunConfig<'_>) {
        let builder = run.builder;

        let test_kind = builder.kind.into();

        builder.ensure(CrateCrabLangdocJsonTypes { host: run.target, test_kind });
    }

    fn run(self, builder: &Builder<'_>) {
        let test_kind = self.test_kind;
        let target = self.host;

        // Use the previous stage compiler to reuse the artifacts that are
        // created when running compiletest for tests/crablangdoc. If this used
        // `compiler`, then it would cause crablangdoc to be built *again*, which
        // isn't really necessary.
        let compiler = builder.compiler_for(builder.top_stage, target, target);
        builder.ensure(compile::CrabLangc::new(compiler, target));

        let mut cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolCrabLangc,
            target,
            test_kind.subcommand(),
            "src/crablangdoc-json-types",
            SourceType::InTree,
            &[],
        );
        if test_kind.subcommand() == "test" && !builder.fail_fast {
            cargo.arg("--no-fail-fast");
        }

        cargo.arg("-p").arg("crablangdoc-json-types");

        cargo.arg("--");
        cargo.args(&builder.config.cmd.test_args());

        if self.host.contains("musl") {
            cargo.arg("'-Ctarget-feature=-crt-static'");
        }

        builder.info(&format!(
            "{} crablangdoc-json-types stage{} ({} -> {})",
            test_kind, compiler.stage, &compiler.host, target
        ));
        let _time = util::timeit(&builder);

        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }
}

/// Some test suites are run inside emulators or on remote devices, and most
/// of our test binaries are linked dynamically which means we need to ship
/// the standard library and such to the emulator ahead of time. This step
/// represents this and is a dependency of all test suites.
///
/// Most of the time this is a no-op. For some steps such as shipping data to
/// QEMU we have to build our own tools so we've got conditional dependencies
/// on those programs as well. Note that the remote test client is built for
/// the build target (us) and the server is built for the target.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RemoteCopyLibs {
    compiler: Compiler,
    target: TargetSelection,
}

impl Step for RemoteCopyLibs {
    type Output = ();

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    fn run(self, builder: &Builder<'_>) {
        let compiler = self.compiler;
        let target = self.target;
        if !builder.remote_tested(target) {
            return;
        }

        builder.ensure(compile::Std::new(compiler, target));

        builder.info(&format!("REMOTE copy libs to emulator ({})", target));

        let server = builder.ensure(tool::RemoteTestServer { compiler, target });

        // Spawn the emulator and wait for it to come online
        let tool = builder.tool_exe(Tool::RemoteTestClient);
        let mut cmd = Command::new(&tool);
        cmd.arg("spawn-emulator").arg(target.triple).arg(&server).arg(builder.tempdir());
        if let Some(rootfs) = builder.qemu_rootfs(target) {
            cmd.arg(rootfs);
        }
        builder.run(&mut cmd);

        // Push all our dylibs to the emulator
        for f in t!(builder.sysroot_libdir(compiler, target).read_dir()) {
            let f = t!(f);
            let name = f.file_name().into_string().unwrap();
            if util::is_dylib(&name) {
                builder.run(Command::new(&tool).arg("push").arg(f.path()));
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Distcheck;

impl Step for Distcheck {
    type Output = ();

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("distcheck")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Distcheck);
    }

    /// Runs "distcheck", a 'make check' from a tarball
    fn run(self, builder: &Builder<'_>) {
        builder.info("Distcheck");
        let dir = builder.tempdir().join("distcheck");
        let _ = fs::remove_dir_all(&dir);
        t!(fs::create_dir_all(&dir));

        // Guarantee that these are built before we begin running.
        builder.ensure(dist::PlainSourceTarball);
        builder.ensure(dist::Src);

        let mut cmd = Command::new("tar");
        cmd.arg("-xf")
            .arg(builder.ensure(dist::PlainSourceTarball).tarball())
            .arg("--strip-components=1")
            .current_dir(&dir);
        builder.run(&mut cmd);
        builder.run(
            Command::new("./configure")
                .args(&builder.config.configure_args)
                .arg("--enable-vendor")
                .current_dir(&dir),
        );
        builder.run(
            Command::new(util::make(&builder.config.build.triple)).arg("check").current_dir(&dir),
        );

        // Now make sure that crablang-src has all of libstd's dependencies
        builder.info("Distcheck crablang-src");
        let dir = builder.tempdir().join("distcheck-src");
        let _ = fs::remove_dir_all(&dir);
        t!(fs::create_dir_all(&dir));

        let mut cmd = Command::new("tar");
        cmd.arg("-xf")
            .arg(builder.ensure(dist::Src).tarball())
            .arg("--strip-components=1")
            .current_dir(&dir);
        builder.run(&mut cmd);

        let toml = dir.join("crablang-src/lib/crablanglib/src/crablang/library/std/Cargo.toml");
        builder.run(
            Command::new(&builder.initial_cargo)
                .arg("generate-lockfile")
                .arg("--manifest-path")
                .arg(&toml)
                .current_dir(&dir),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Bootstrap;

impl Step for Bootstrap {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    /// Tests the build system itself.
    fn run(self, builder: &Builder<'_>) {
        let mut check_bootstrap = Command::new(&builder.python());
        check_bootstrap.arg("bootstrap_test.py").current_dir(builder.src.join("src/bootstrap/"));
        try_run(builder, &mut check_bootstrap);

        let mut cmd = Command::new(&builder.initial_cargo);
        cmd.arg("test")
            .current_dir(builder.src.join("src/bootstrap"))
            .env("CRABLANGFLAGS", "-Cdebuginfo=2")
            .env("CARGO_TARGET_DIR", builder.out.join("bootstrap"))
            .env("CRABLANGC_BOOTSTRAP", "1")
            .env("CRABLANGDOC", builder.crablangdoc(builder.compiler(0, builder.build.build)))
            .env("CRABLANGC", &builder.initial_crablangc);
        if let Some(flags) = option_env!("CRABLANGFLAGS") {
            // Use the same crablangc flags for testing as for "normal" compilation,
            // so that Cargo doesn’t recompile the entire dependency graph every time:
            // https://github.com/crablang/crablang/issues/49215
            cmd.env("CRABLANGFLAGS", flags);
        }
        if !builder.fail_fast {
            cmd.arg("--no-fail-fast");
        }
        match builder.doc_tests {
            DocTests::Only => {
                cmd.arg("--doc");
            }
            DocTests::No => {
                cmd.args(&["--lib", "--bins", "--examples", "--tests", "--benches"]);
            }
            DocTests::Yes => {}
        }

        cmd.arg("--").args(&builder.config.cmd.test_args());
        // crablangbuild tests are racy on directory creation so just run them one at a time.
        // Since there's not many this shouldn't be a problem.
        cmd.arg("--test-threads=1");
        add_flags_and_try_run_tests(builder, &mut cmd);
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/bootstrap")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Bootstrap);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TierCheck {
    pub compiler: Compiler,
}

impl Step for TierCheck {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/tier-check")
    }

    fn make_run(run: RunConfig<'_>) {
        let compiler =
            run.builder.compiler_for(run.builder.top_stage, run.builder.build.build, run.target);
        run.builder.ensure(TierCheck { compiler });
    }

    /// Tests the Platform Support page in the crablangc book.
    fn run(self, builder: &Builder<'_>) {
        builder.ensure(compile::Std::new(self.compiler, self.compiler.host));
        let mut cargo = tool::prepare_tool_cargo(
            builder,
            self.compiler,
            Mode::ToolStd,
            self.compiler.host,
            "run",
            "src/tools/tier-check",
            SourceType::InTree,
            &[],
        );
        cargo.arg(builder.src.join("src/doc/crablangc/src/platform-support.md"));
        cargo.arg(&builder.crablangc(self.compiler));
        if builder.is_verbose() {
            cargo.arg("--verbose");
        }

        builder.info("platform support check");
        try_run(builder, &mut cargo.into());
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ReplacePlaceholderTest;

impl Step for ReplacePlaceholderTest {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    /// Ensure the version placeholder replacement tool builds
    fn run(self, builder: &Builder<'_>) {
        builder.info("build check for version replacement placeholder");

        // Test the version placeholder replacement tool itself.
        let bootstrap_host = builder.config.build;
        let compiler = builder.compiler(0, bootstrap_host);
        let cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolBootstrap,
            bootstrap_host,
            "test",
            "src/tools/replace-version-placeholder",
            SourceType::InTree,
            &[],
        );
        add_flags_and_try_run_tests(builder, &mut cargo.into());
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/replace-version-placeholder")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Self);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LintDocs {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for LintDocs {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/lint-docs")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(LintDocs {
            compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
            target: run.target,
        });
    }

    /// Tests that the lint examples in the crablangc book generate the correct
    /// lints and have the expected format.
    fn run(self, builder: &Builder<'_>) {
        builder.ensure(crate::doc::CrabLangcBook {
            compiler: self.compiler,
            target: self.target,
            validate: true,
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangInstaller;

impl Step for CrabLangInstaller {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    /// Ensure the version placeholder replacement tool builds
    fn run(self, builder: &Builder<'_>) {
        builder.info("test crablang-installer");

        let bootstrap_host = builder.config.build;
        let compiler = builder.compiler(0, bootstrap_host);
        let cargo = tool::prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolBootstrap,
            bootstrap_host,
            "test",
            "src/tools/crablang-installer",
            SourceType::InTree,
            &[],
        );
        try_run(builder, &mut cargo.into());

        // We currently don't support running the test.sh script outside linux(?) environments.
        // Eventually this should likely migrate to #[test]s in crablang-installer proper rather than a
        // set of scripts, which will likely allow dropping this if.
        if bootstrap_host != "x86_64-unknown-linux-gnu" {
            return;
        }

        let mut cmd =
            std::process::Command::new(builder.src.join("src/tools/crablang-installer/test.sh"));
        let tmpdir = testdir(builder, compiler.host).join("crablang-installer");
        let _ = std::fs::remove_dir_all(&tmpdir);
        let _ = std::fs::create_dir_all(&tmpdir);
        cmd.current_dir(&tmpdir);
        cmd.env("CARGO_TARGET_DIR", tmpdir.join("cargo-target"));
        cmd.env("CARGO", &builder.initial_cargo);
        cmd.env("CRABLANGC", &builder.initial_crablangc);
        cmd.env("TMP_DIR", &tmpdir);
        try_run(builder, &mut cmd);
    }

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablang-installer")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Self);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TestHelpers {
    pub target: TargetSelection,
}

impl Step for TestHelpers {
    type Output = ();

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("tests/auxiliary/crablang_test_helpers.c")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(TestHelpers { target: run.target })
    }

    /// Compiles the `crablang_test_helpers.c` library which we used in various
    /// `run-pass` tests for ABI testing.
    fn run(self, builder: &Builder<'_>) {
        if builder.config.dry_run() {
            return;
        }
        // The x86_64-fortanix-unknown-sgx target doesn't have a working C
        // toolchain. However, some x86_64 ELF objects can be linked
        // without issues. Use this hack to compile the test helpers.
        let target = if self.target == "x86_64-fortanix-unknown-sgx" {
            TargetSelection::from_user("x86_64-unknown-linux-gnu")
        } else {
            self.target
        };
        let dst = builder.test_helpers_out(target);
        let src = builder.src.join("tests/auxiliary/crablang_test_helpers.c");
        if up_to_date(&src, &dst.join("libcrablang_test_helpers.a")) {
            return;
        }

        builder.info("Building test helpers");
        t!(fs::create_dir_all(&dst));
        let mut cfg = cc::Build::new();
        // FIXME: Workaround for https://github.com/emscripten-core/emscripten/issues/9013
        if target.contains("emscripten") {
            cfg.pic(false);
        }

        // We may have found various cross-compilers a little differently due to our
        // extra configuration, so inform cc of these compilers. Note, though, that
        // on MSVC we still need cc's detection of env vars (ugh).
        if !target.contains("msvc") {
            if let Some(ar) = builder.ar(target) {
                cfg.archiver(ar);
            }
            cfg.compiler(builder.cc(target));
        }
        cfg.cargo_metadata(false)
            .out_dir(&dst)
            .target(&target.triple)
            .host(&builder.config.build.triple)
            .opt_level(0)
            .warnings(false)
            .debug(false)
            .file(builder.src.join("tests/auxiliary/crablang_test_helpers.c"))
            .compile("crablang_test_helpers");
    }
}
