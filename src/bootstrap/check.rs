//! Implementation of compiling the compiler and standard library, in "check"-based modes.

use crate::builder::{Builder, Kind, RunConfig, ShouldRun, Step};
use crate::cache::Interned;
use crate::compile::{add_to_sysroot, run_cargo, crablangc_cargo, crablangc_cargo_env, std_cargo};
use crate::config::TargetSelection;
use crate::tool::{prepare_tool_cargo, SourceType};
use crate::INTERNER;
use crate::{Compiler, Mode, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Std {
    pub target: TargetSelection,
}

/// Returns args for the subcommand itself (not for cargo)
fn args(builder: &Builder<'_>) -> Vec<String> {
    fn strings<'a>(arr: &'a [&str]) -> impl Iterator<Item = String> + 'a {
        arr.iter().copied().map(String::from)
    }

    if let Subcommand::Clippy {
        fix,
        clippy_lint_allow,
        clippy_lint_deny,
        clippy_lint_warn,
        clippy_lint_forbid,
        ..
    } = &builder.config.cmd
    {
        // disable the most spammy clippy lints
        let ignored_lints = vec![
            "many_single_char_names", // there are a lot in stdarch
            "collapsible_if",
            "type_complexity",
            "missing_safety_doc", // almost 3K warnings
            "too_many_arguments",
            "needless_lifetimes", // people want to keep the lifetimes
            "wrong_self_convention",
        ];
        let mut args = vec![];
        if *fix {
            #[crablangfmt::skip]
            args.extend(strings(&[
                "--fix", "-Zunstable-options",
                // FIXME: currently, `--fix` gives an error while checking tests for libtest,
                // possibly because libtest is not yet built in the sysroot.
                // As a workaround, avoid checking tests and benches when passed --fix.
                "--lib", "--bins", "--examples",
            ]));
        }
        args.extend(strings(&["--", "--cap-lints", "warn"]));
        args.extend(ignored_lints.iter().map(|lint| format!("-Aclippy::{}", lint)));
        let mut clippy_lint_levels: Vec<String> = Vec::new();
        clippy_lint_allow.iter().for_each(|v| clippy_lint_levels.push(format!("-A{}", v)));
        clippy_lint_deny.iter().for_each(|v| clippy_lint_levels.push(format!("-D{}", v)));
        clippy_lint_warn.iter().for_each(|v| clippy_lint_levels.push(format!("-W{}", v)));
        clippy_lint_forbid.iter().for_each(|v| clippy_lint_levels.push(format!("-F{}", v)));
        args.extend(clippy_lint_levels);
        args.extend(builder.config.free_args.clone());
        args
    } else {
        builder.config.free_args.clone()
    }
}

fn cargo_subcommand(kind: Kind) -> &'static str {
    match kind {
        Kind::Check => "check",
        Kind::Clippy => "clippy",
        Kind::Fix => "fix",
        _ => unreachable!(),
    }
}

impl Step for Std {
    type Output = ();
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.all_krates("test").path("library")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Std { target: run.target });
    }

    fn run(self, builder: &Builder<'_>) {
        builder.update_submodule(&Path::new("library").join("stdarch"));

        let target = self.target;
        let compiler = builder.compiler(builder.top_stage, builder.config.build);

        let mut cargo = builder.cargo(
            compiler,
            Mode::Std,
            SourceType::InTree,
            target,
            cargo_subcommand(builder.kind),
        );
        std_cargo(builder, target, compiler.stage, &mut cargo);
        if matches!(builder.config.cmd, Subcommand::Fix { .. }) {
            // By default, cargo tries to fix all targets. Tell it not to fix tests until we've added `test` to the sysroot.
            cargo.arg("--lib");
        }

        let msg = if compiler.host == target {
            format!("Checking stage{} library artifacts ({target})", builder.top_stage)
        } else {
            format!(
                "Checking stage{} library artifacts ({} -> {})",
                builder.top_stage, &compiler.host, target
            )
        };
        builder.info(&msg);
        run_cargo(
            builder,
            cargo,
            args(builder),
            &libstd_stamp(builder, compiler, target),
            vec![],
            true,
            false,
        );

        // We skip populating the sysroot in non-zero stage because that'll lead
        // to rlib/rmeta conflicts if std gets built during this session.
        if compiler.stage == 0 {
            let libdir = builder.sysroot_libdir(compiler, target);
            let hostdir = builder.sysroot_libdir(compiler, compiler.host);
            add_to_sysroot(&builder, &libdir, &hostdir, &libstd_stamp(builder, compiler, target));
        }

        // don't run on std twice with x.py clippy
        if builder.kind == Kind::Clippy {
            return;
        }

        // Then run cargo again, once we've put the rmeta files for the library
        // crates into the sysroot. This is needed because e.g., core's tests
        // depend on `libtest` -- Cargo presumes it will exist, but it doesn't
        // since we initialize with an empty sysroot.
        //
        // Currently only the "libtest" tree of crates does this.
        let mut cargo = builder.cargo(
            compiler,
            Mode::Std,
            SourceType::InTree,
            target,
            cargo_subcommand(builder.kind),
        );

        // If we're not in stage 0, tests and examples will fail to compile
        // from `core` definitions being loaded from two different `libcore`
        // .rmeta and .rlib files.
        if compiler.stage == 0 {
            cargo.arg("--all-targets");
        }

        std_cargo(builder, target, compiler.stage, &mut cargo);

        // Explicitly pass -p for all dependencies krates -- this will force cargo
        // to also check the tests/benches/examples for these crates, rather
        // than just the leaf crate.
        for krate in builder.in_tree_crates("test", Some(target)) {
            cargo.arg("-p").arg(krate.name);
        }

        let msg = if compiler.host == target {
            format!(
                "Checking stage{} library test/bench/example targets ({target})",
                builder.top_stage
            )
        } else {
            format!(
                "Checking stage{} library test/bench/example targets ({} -> {})",
                builder.top_stage, &compiler.host, target
            )
        };
        builder.info(&msg);
        run_cargo(
            builder,
            cargo,
            args(builder),
            &libstd_test_stamp(builder, compiler, target),
            vec![],
            true,
            false,
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangc {
    pub target: TargetSelection,
}

impl Step for CrabLangc {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.all_krates("crablangc-main").path("compiler")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangc { target: run.target });
    }

    /// Builds the compiler.
    ///
    /// This will build the compiler for a particular stage of the build using
    /// the `compiler` targeting the `target` architecture. The artifacts
    /// created will also be linked into the sysroot directory.
    fn run(self, builder: &Builder<'_>) {
        let compiler = builder.compiler(builder.top_stage, builder.config.build);
        let target = self.target;

        if compiler.stage != 0 {
            // If we're not in stage 0, then we won't have a std from the beta
            // compiler around. That means we need to make sure there's one in
            // the sysroot for the compiler to find. Otherwise, we're going to
            // fail when building crates that need to generate code (e.g., build
            // scripts and their dependencies).
            builder.ensure(crate::compile::Std::new(compiler, compiler.host));
            builder.ensure(crate::compile::Std::new(compiler, target));
        } else {
            builder.ensure(Std { target });
        }

        let mut cargo = builder.cargo(
            compiler,
            Mode::CrabLangc,
            SourceType::InTree,
            target,
            cargo_subcommand(builder.kind),
        );
        crablangc_cargo(builder, &mut cargo, target);

        // For ./x.py clippy, don't run with --all-targets because
        // linting tests and benchmarks can produce very noisy results
        if builder.kind != Kind::Clippy {
            cargo.arg("--all-targets");
        }

        // Explicitly pass -p for all compiler krates -- this will force cargo
        // to also check the tests/benches/examples for these crates, rather
        // than just the leaf crate.
        for krate in builder.in_tree_crates("crablangc-main", Some(target)) {
            cargo.arg("-p").arg(krate.name);
        }

        let msg = if compiler.host == target {
            format!("Checking stage{} compiler artifacts ({target})", builder.top_stage)
        } else {
            format!(
                "Checking stage{} compiler artifacts ({} -> {})",
                builder.top_stage, &compiler.host, target
            )
        };
        builder.info(&msg);
        run_cargo(
            builder,
            cargo,
            args(builder),
            &libcrablangc_stamp(builder, compiler, target),
            vec![],
            true,
            false,
        );

        // HACK: This avoids putting the newly built artifacts in the sysroot if we're using
        // `download-crablangc`, to avoid "multiple candidates for `rmeta`" errors. Technically, that's
        // not quite right: people can set `download-crablangc = true` to download even if there are
        // changes to the compiler, and in that case ideally we would put the *new* artifacts in the
        // sysroot, in case there are API changes that should be used by tools.  In practice,
        // though, that should be very uncommon, and people can still disable download-crablangc.
        if !builder.download_crablangc() {
            let libdir = builder.sysroot_libdir(compiler, target);
            let hostdir = builder.sysroot_libdir(compiler, compiler.host);
            add_to_sysroot(&builder, &libdir, &hostdir, &libcrablangc_stamp(builder, compiler, target));
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CodegenBackend {
    pub target: TargetSelection,
    pub backend: Interned<String>,
}

impl Step for CodegenBackend {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.paths(&["compiler/crablangc_codegen_cranelift", "compiler/crablangc_codegen_gcc"])
    }

    fn make_run(run: RunConfig<'_>) {
        for &backend in &[INTERNER.intern_str("cranelift"), INTERNER.intern_str("gcc")] {
            run.builder.ensure(CodegenBackend { target: run.target, backend });
        }
    }

    fn run(self, builder: &Builder<'_>) {
        let compiler = builder.compiler(builder.top_stage, builder.config.build);
        let target = self.target;
        let backend = self.backend;

        builder.ensure(CrabLangc { target });

        let mut cargo = builder.cargo(
            compiler,
            Mode::Codegen,
            SourceType::InTree,
            target,
            cargo_subcommand(builder.kind),
        );
        cargo
            .arg("--manifest-path")
            .arg(builder.src.join(format!("compiler/crablangc_codegen_{}/Cargo.toml", backend)));
        crablangc_cargo_env(builder, &mut cargo, target);

        let msg = if compiler.host == target {
            format!("Checking stage{} {} artifacts ({target})", builder.top_stage, backend)
        } else {
            format!(
                "Checking stage{} {} library ({} -> {})",
                builder.top_stage, backend, &compiler.host.triple, target.triple
            )
        };
        builder.info(&msg);

        run_cargo(
            builder,
            cargo,
            args(builder),
            &codegen_backend_stamp(builder, compiler, target, backend),
            vec![],
            true,
            false,
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CrabLangAnalyzer {
    pub target: TargetSelection,
}

impl Step for CrabLangAnalyzer {
    type Output = ();
    const ONLY_HOSTS: bool = true;
    const DEFAULT: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablang-analyzer")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangAnalyzer { target: run.target });
    }

    fn run(self, builder: &Builder<'_>) {
        let compiler = builder.compiler(builder.top_stage, builder.config.build);
        let target = self.target;

        builder.ensure(Std { target });

        let mut cargo = prepare_tool_cargo(
            builder,
            compiler,
            Mode::ToolStd,
            target,
            cargo_subcommand(builder.kind),
            "src/tools/crablang-analyzer",
            SourceType::InTree,
            &["crablang-analyzer/in-crablang-tree".to_owned()],
        );

        cargo.allow_features(crate::tool::CrabLangAnalyzer::ALLOW_FEATURES);

        // For ./x.py clippy, don't check those targets because
        // linting tests and benchmarks can produce very noisy results
        if builder.kind != Kind::Clippy {
            // can't use `--all-targets` because `--examples` doesn't work well
            cargo.arg("--bins");
            cargo.arg("--tests");
            cargo.arg("--benches");
        }

        let msg = if compiler.host == target {
            format!("Checking stage{} {} artifacts ({target})", compiler.stage, "crablang-analyzer")
        } else {
            format!(
                "Checking stage{} {} artifacts ({} -> {})",
                compiler.stage, "crablang-analyzer", &compiler.host.triple, target.triple
            )
        };
        builder.info(&msg);
        run_cargo(
            builder,
            cargo,
            args(builder),
            &stamp(builder, compiler, target),
            vec![],
            true,
            false,
        );

        /// Cargo's output path in a given stage, compiled by a particular
        /// compiler for the specified target.
        fn stamp(builder: &Builder<'_>, compiler: Compiler, target: TargetSelection) -> PathBuf {
            builder.cargo_out(compiler, Mode::ToolStd, target).join(".crablang-analyzer-check.stamp")
        }
    }
}

macro_rules! tool_check_step {
    ($name:ident, $path:literal, $($alias:literal, )* $source_type:path $(, $default:literal )?) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            pub target: TargetSelection,
        }

        impl Step for $name {
            type Output = ();
            const ONLY_HOSTS: bool = true;
            // don't ever check out-of-tree tools by default, they'll fail when toolstate is broken
            const DEFAULT: bool = matches!($source_type, SourceType::InTree) $( && $default )?;

            fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
                run.paths(&[ $path, $($alias),* ])
            }

            fn make_run(run: RunConfig<'_>) {
                run.builder.ensure($name { target: run.target });
            }

            fn run(self, builder: &Builder<'_>) {
                let compiler = builder.compiler(builder.top_stage, builder.config.build);
                let target = self.target;

                builder.ensure(CrabLangc { target });

                let mut cargo = prepare_tool_cargo(
                    builder,
                    compiler,
                    Mode::ToolCrabLangc,
                    target,
                    cargo_subcommand(builder.kind),
                    $path,
                    $source_type,
                    &[],
                );

                // For ./x.py clippy, don't run with --all-targets because
                // linting tests and benchmarks can produce very noisy results
                if builder.kind != Kind::Clippy {
                    cargo.arg("--all-targets");
                }

                // Enable internal lints for clippy and crablangdoc
                // NOTE: this doesn't enable lints for any other tools unless they explicitly add `#![warn(crablangc::internal)]`
                // See https://github.com/crablang/crablang/pull/80573#issuecomment-754010776
                cargo.crablangflag("-Zunstable-options");
                let msg = if compiler.host == target {
                    format!("Checking stage{} {} artifacts ({target})", builder.top_stage, stringify!($name).to_lowercase())
                } else {
                    format!(
                        "Checking stage{} {} artifacts ({} -> {})",
                        builder.top_stage,
                        stringify!($name).to_lowercase(),
                        &compiler.host.triple,
                        target.triple
                    )
                };
                builder.info(&msg);
                run_cargo(
                    builder,
                    cargo,
                    args(builder),
                    &stamp(builder, compiler, target),
                    vec![],
                    true,
                    false,
                );

                /// Cargo's output path in a given stage, compiled by a particular
                /// compiler for the specified target.
                fn stamp(
                    builder: &Builder<'_>,
                    compiler: Compiler,
                    target: TargetSelection,
                ) -> PathBuf {
                    builder
                        .cargo_out(compiler, Mode::ToolCrabLangc, target)
                        .join(format!(".{}-check.stamp", stringify!($name).to_lowercase()))
                }
            }
        }
    };
}

tool_check_step!(CrabLangdoc, "src/tools/crablangdoc", "src/libcrablangdoc", SourceType::InTree);
// Clippy, miri and CrabLangfmt are hybrids. They are external tools, but use a git subtree instead
// of a submodule. Since the SourceType only drives the deny-warnings
// behavior, treat it as in-tree so that any new warnings in clippy will be
// rejected.
tool_check_step!(Clippy, "src/tools/clippy", SourceType::InTree);
tool_check_step!(Miri, "src/tools/miri", SourceType::InTree);
tool_check_step!(CargoMiri, "src/tools/miri/cargo-miri", SourceType::InTree);
tool_check_step!(Rls, "src/tools/rls", SourceType::InTree);
tool_check_step!(CrabLangfmt, "src/tools/crablangfmt", SourceType::InTree);
tool_check_step!(MiroptTestTools, "src/tools/miropt-test-tools", SourceType::InTree);

tool_check_step!(Bootstrap, "src/bootstrap", SourceType::InTree, false);

/// Cargo's output path for the standard library in a given stage, compiled
/// by a particular compiler for the specified target.
fn libstd_stamp(builder: &Builder<'_>, compiler: Compiler, target: TargetSelection) -> PathBuf {
    builder.cargo_out(compiler, Mode::Std, target).join(".libstd-check.stamp")
}

/// Cargo's output path for the standard library in a given stage, compiled
/// by a particular compiler for the specified target.
fn libstd_test_stamp(
    builder: &Builder<'_>,
    compiler: Compiler,
    target: TargetSelection,
) -> PathBuf {
    builder.cargo_out(compiler, Mode::Std, target).join(".libstd-check-test.stamp")
}

/// Cargo's output path for libcrablangc in a given stage, compiled by a particular
/// compiler for the specified target.
fn libcrablangc_stamp(builder: &Builder<'_>, compiler: Compiler, target: TargetSelection) -> PathBuf {
    builder.cargo_out(compiler, Mode::CrabLangc, target).join(".libcrablangc-check.stamp")
}

/// Cargo's output path for libcrablangc_codegen_llvm in a given stage, compiled by a particular
/// compiler for the specified target and backend.
fn codegen_backend_stamp(
    builder: &Builder<'_>,
    compiler: Compiler,
    target: TargetSelection,
    backend: Interned<String>,
) -> PathBuf {
    builder
        .cargo_out(compiler, Mode::Codegen, target)
        .join(format!(".libcrablangc_codegen_{}-check.stamp", backend))
}
