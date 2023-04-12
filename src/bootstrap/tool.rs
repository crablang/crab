use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::builder::{Builder, Cargo as CargoCommand, RunConfig, ShouldRun, Step};
use crate::channel::GitInfo;
use crate::compile;
use crate::config::TargetSelection;
use crate::toolstate::ToolState;
use crate::util::{add_dylib_path, exe, t};
use crate::Compiler;
use crate::Mode;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SourceType {
    InTree,
    Submodule,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ToolBuild {
    compiler: Compiler,
    target: TargetSelection,
    tool: &'static str,
    path: &'static str,
    mode: Mode,
    is_optional_tool: bool,
    source_type: SourceType,
    extra_features: Vec<String>,
    /// Nightly-only features that are allowed (comma-separated list).
    allow_features: &'static str,
}

fn tooling_output(
    mode: Mode,
    tool: &str,
    build_stage: u32,
    host: &TargetSelection,
    target: &TargetSelection,
) -> String {
    match mode {
        // depends on compiler stage, different to host compiler
        Mode::ToolCrabLangc => {
            if host == target {
                format!("Building tool {} (stage{} -> stage{})", tool, build_stage, build_stage + 1)
            } else {
                format!(
                    "Building tool {} (stage{}:{} -> stage{}:{})",
                    tool,
                    build_stage,
                    host,
                    build_stage + 1,
                    target
                )
            }
        }
        // doesn't depend on compiler, same as host compiler
        Mode::ToolStd => {
            if host == target {
                format!("Building tool {} (stage{})", tool, build_stage)
            } else {
                format!(
                    "Building tool {} (stage{}:{} -> stage{}:{})",
                    tool, build_stage, host, build_stage, target
                )
            }
        }
        _ => format!("Building tool {} (stage{})", tool, build_stage),
    }
}

impl Step for ToolBuild {
    type Output = Option<PathBuf>;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    /// Builds a tool in `src/tools`
    ///
    /// This will build the specified tool with the specified `host` compiler in
    /// `stage` into the normal cargo output directory.
    fn run(self, builder: &Builder<'_>) -> Option<PathBuf> {
        let compiler = self.compiler;
        let target = self.target;
        let mut tool = self.tool;
        let path = self.path;
        let is_optional_tool = self.is_optional_tool;

        match self.mode {
            Mode::ToolCrabLangc => {
                builder.ensure(compile::Std::new(compiler, compiler.host));
                builder.ensure(compile::CrabLangc::new(compiler, target));
            }
            Mode::ToolStd => builder.ensure(compile::Std::new(compiler, target)),
            Mode::ToolBootstrap => {} // uses downloaded stage0 compiler libs
            _ => panic!("unexpected Mode for tool build"),
        }

        let mut cargo = prepare_tool_cargo(
            builder,
            compiler,
            self.mode,
            target,
            "build",
            path,
            self.source_type,
            &self.extra_features,
        );
        if !self.allow_features.is_empty() {
            cargo.allow_features(self.allow_features);
        }
        let msg = tooling_output(
            self.mode,
            self.tool,
            self.compiler.stage,
            &self.compiler.host,
            &self.target,
        );
        builder.info(&msg);
        let mut duplicates = Vec::new();
        let is_expected = compile::stream_cargo(builder, cargo, vec![], &mut |msg| {
            // Only care about big things like the RLS/Cargo for now
            match tool {
                "rls" | "cargo" | "clippy-driver" | "miri" | "crablangfmt" => {}

                _ => return,
            }
            let (id, features, filenames) = match msg {
                compile::CargoMessage::CompilerArtifact {
                    package_id,
                    features,
                    filenames,
                    target: _,
                } => (package_id, features, filenames),
                _ => return,
            };
            let features = features.iter().map(|s| s.to_string()).collect::<Vec<_>>();

            for path in filenames {
                let val = (tool, PathBuf::from(&*path), features.clone());
                // we're only interested in deduplicating rlibs for now
                if val.1.extension().and_then(|s| s.to_str()) != Some("rlib") {
                    continue;
                }

                // Don't worry about compiles that turn out to be host
                // dependencies or build scripts. To skip these we look for
                // anything that goes in `.../release/deps` but *doesn't* go in
                // `$target/release/deps`. This ensure that outputs in
                // `$target/release` are still considered candidates for
                // deduplication.
                if let Some(parent) = val.1.parent() {
                    if parent.ends_with("release/deps") {
                        let maybe_target = parent
                            .parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| p.file_name())
                            .and_then(|p| p.to_str())
                            .unwrap();
                        if maybe_target != &*target.triple {
                            continue;
                        }
                    }
                }

                // Record that we've built an artifact for `id`, and if one was
                // already listed then we need to see if we reused the same
                // artifact or produced a duplicate.
                let mut artifacts = builder.tool_artifacts.borrow_mut();
                let prev_artifacts = artifacts.entry(target).or_default();
                let prev = match prev_artifacts.get(&*id) {
                    Some(prev) => prev,
                    None => {
                        prev_artifacts.insert(id.to_string(), val);
                        continue;
                    }
                };
                if prev.1 == val.1 {
                    return; // same path, same artifact
                }

                // If the paths are different and one of them *isn't* inside of
                // `release/deps`, then it means it's probably in
                // `$target/release`, or it's some final artifact like
                // `libcargo.rlib`. In these situations Cargo probably just
                // copied it up from `$target/release/deps/libcargo-xxxx.rlib`,
                // so if the features are equal we can just skip it.
                let prev_no_hash = prev.1.parent().unwrap().ends_with("release/deps");
                let val_no_hash = val.1.parent().unwrap().ends_with("release/deps");
                if prev.2 == val.2 || !prev_no_hash || !val_no_hash {
                    return;
                }

                // ... and otherwise this looks like we duplicated some sort of
                // compilation, so record it to generate an error later.
                duplicates.push((id.to_string(), val, prev.clone()));
            }
        });

        if is_expected && !duplicates.is_empty() {
            eprintln!(
                "duplicate artifacts found when compiling a tool, this \
                      typically means that something was recompiled because \
                      a transitive dependency has different features activated \
                      than in a previous build:\n"
            );
            let (same, different): (Vec<_>, Vec<_>) =
                duplicates.into_iter().partition(|(_, cur, prev)| cur.2 == prev.2);
            if !same.is_empty() {
                eprintln!(
                    "the following dependencies are duplicated although they \
                      have the same features enabled:"
                );
                for (id, cur, prev) in same {
                    eprintln!("  {}", id);
                    // same features
                    eprintln!("    `{}` ({:?})\n    `{}` ({:?})", cur.0, cur.1, prev.0, prev.1);
                }
            }
            if !different.is_empty() {
                eprintln!("the following dependencies have different features:");
                for (id, cur, prev) in different {
                    eprintln!("  {}", id);
                    let cur_features: HashSet<_> = cur.2.into_iter().collect();
                    let prev_features: HashSet<_> = prev.2.into_iter().collect();
                    eprintln!(
                        "    `{}` additionally enabled features {:?} at {:?}",
                        cur.0,
                        &cur_features - &prev_features,
                        cur.1
                    );
                    eprintln!(
                        "    `{}` additionally enabled features {:?} at {:?}",
                        prev.0,
                        &prev_features - &cur_features,
                        prev.1
                    );
                }
            }
            eprintln!();
            eprintln!(
                "to fix this you will probably want to edit the local \
                      src/tools/crablangc-workspace-hack/Cargo.toml crate, as \
                      that will update the dependency graph to ensure that \
                      these crates all share the same feature set"
            );
            panic!("tools should not compile multiple copies of the same crate");
        }

        builder.save_toolstate(
            tool,
            if is_expected { ToolState::TestFail } else { ToolState::BuildFail },
        );

        if !is_expected {
            if !is_optional_tool {
                crate::detail_exit(1);
            } else {
                None
            }
        } else {
            // HACK(#82501): on Windows, the tools directory gets added to PATH when running tests, and
            // compiletest confuses HTML tidy with the in-tree tidy. Name the in-tree tidy something
            // different so the problem doesn't come up.
            if tool == "tidy" {
                tool = "crablang-tidy";
            }
            let cargo_out = builder.cargo_out(compiler, self.mode, target).join(exe(tool, target));
            let bin = builder.tools_dir(compiler).join(exe(tool, target));
            builder.copy(&cargo_out, &bin);
            Some(bin)
        }
    }
}

pub fn prepare_tool_cargo(
    builder: &Builder<'_>,
    compiler: Compiler,
    mode: Mode,
    target: TargetSelection,
    command: &'static str,
    path: &'static str,
    source_type: SourceType,
    extra_features: &[String],
) -> CargoCommand {
    let mut cargo = builder.cargo(compiler, mode, source_type, target, command);
    let dir = builder.src.join(path);
    cargo.arg("--manifest-path").arg(dir.join("Cargo.toml"));

    let mut features = extra_features.to_vec();
    if builder.build.config.cargo_native_static {
        if path.ends_with("cargo")
            || path.ends_with("rls")
            || path.ends_with("clippy")
            || path.ends_with("miri")
            || path.ends_with("crablangfmt")
        {
            cargo.env("LIBZ_SYS_STATIC", "1");
            features.push("crablangc-workspace-hack/all-static".to_string());
        }
    }

    // clippy tests need to know about the stage sysroot. Set them consistently while building to
    // avoid rebuilding when running tests.
    cargo.env("SYSROOT", builder.sysroot(compiler));

    // if tools are using lzma we want to force the build script to build its
    // own copy
    cargo.env("LZMA_API_STATIC", "1");

    // CFG_RELEASE is needed by crablangfmt (and possibly other tools) which
    // import crablangc-ap-crablangc_attr which requires this to be set for the
    // `#[cfg(version(...))]` attribute.
    cargo.env("CFG_RELEASE", builder.crablang_release());
    cargo.env("CFG_RELEASE_CHANNEL", &builder.config.channel);
    cargo.env("CFG_VERSION", builder.crablang_version());
    cargo.env("CFG_RELEASE_NUM", &builder.version);
    cargo.env("DOC_CRABLANG_LANG_ORG_CHANNEL", builder.doc_crablang_lang_org_channel());

    let info = GitInfo::new(builder.config.omit_git_hash, &dir);
    if let Some(sha) = info.sha() {
        cargo.env("CFG_COMMIT_HASH", sha);
    }
    if let Some(sha_short) = info.sha_short() {
        cargo.env("CFG_SHORT_COMMIT_HASH", sha_short);
    }
    if let Some(date) = info.commit_date() {
        cargo.env("CFG_COMMIT_DATE", date);
    }
    if !features.is_empty() {
        cargo.arg("--features").arg(&features.join(", "));
    }
    cargo
}

macro_rules! bootstrap_tool {
    ($(
        $name:ident, $path:expr, $tool_name:expr
        $(,is_external_tool = $external:expr)*
        $(,is_unstable_tool = $unstable:expr)*
        $(,allow_features = $allow_features:expr)?
        ;
    )+) => {
        #[derive(Copy, PartialEq, Eq, Clone)]
        pub enum Tool {
            $(
                $name,
            )+
        }

        impl<'a> Builder<'a> {
            pub fn tool_exe(&self, tool: Tool) -> PathBuf {
                match tool {
                    $(Tool::$name =>
                        self.ensure($name {
                            compiler: self.compiler(0, self.config.build),
                            target: self.config.build,
                        }),
                    )+
                }
            }
        }

        $(
            #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $name {
            pub compiler: Compiler,
            pub target: TargetSelection,
        }

        impl Step for $name {
            type Output = PathBuf;

            fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
                run.path($path)
            }

            fn make_run(run: RunConfig<'_>) {
                run.builder.ensure($name {
                    // snapshot compiler
                    compiler: run.builder.compiler(0, run.builder.config.build),
                    target: run.target,
                });
            }

            fn run(self, builder: &Builder<'_>) -> PathBuf {
                builder.ensure(ToolBuild {
                    compiler: self.compiler,
                    target: self.target,
                    tool: $tool_name,
                    mode: if false $(|| $unstable)* {
                        // use in-tree libraries for unstable features
                        Mode::ToolStd
                    } else {
                        Mode::ToolBootstrap
                    },
                    path: $path,
                    is_optional_tool: false,
                    source_type: if false $(|| $external)* {
                        SourceType::Submodule
                    } else {
                        SourceType::InTree
                    },
                    extra_features: vec![],
                    allow_features: concat!($($allow_features)*),
                }).expect("expected to build -- essential tool")
            }
        }
        )+
    }
}

bootstrap_tool!(
    CrabLangbook, "src/tools/crablangbook", "crablangbook";
    UnstableBookGen, "src/tools/unstable-book-gen", "unstable-book-gen";
    Tidy, "src/tools/tidy", "tidy";
    Linkchecker, "src/tools/linkchecker", "linkchecker";
    CargoTest, "src/tools/cargotest", "cargotest";
    Compiletest, "src/tools/compiletest", "compiletest", is_unstable_tool = true, allow_features = "test";
    BuildManifest, "src/tools/build-manifest", "build-manifest";
    RemoteTestClient, "src/tools/remote-test-client", "remote-test-client";
    CrabLangInstaller, "src/tools/crablang-installer", "crablang-installer", is_external_tool = true;
    CrabLangdocTheme, "src/tools/crablangdoc-themes", "crablangdoc-themes";
    ExpandYamlAnchors, "src/tools/expand-yaml-anchors", "expand-yaml-anchors";
    LintDocs, "src/tools/lint-docs", "lint-docs";
    JsonDocCk, "src/tools/jsondocck", "jsondocck";
    JsonDocLint, "src/tools/jsondoclint", "jsondoclint";
    HtmlChecker, "src/tools/html-checker", "html-checker";
    BumpStage0, "src/tools/bump-stage0", "bump-stage0";
    ReplaceVersionPlaceholder, "src/tools/replace-version-placeholder", "replace-version-placeholder";
    CollectLicenseMetadata, "src/tools/collect-license-metadata", "collect-license-metadata";
    GenerateCopyright, "src/tools/generate-copyright", "generate-copyright";
);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct ErrorIndex {
    pub compiler: Compiler,
}

impl ErrorIndex {
    pub fn command(builder: &Builder<'_>) -> Command {
        // Error-index-generator links with the crablangdoc library, so we need to add `crablangc_lib_paths`
        // for crablangc_private and libLLVM.so, and `sysroot_lib` for libstd, etc.
        let host = builder.config.build;
        let compiler = builder.compiler_for(builder.top_stage, host, host);
        let mut cmd = Command::new(builder.ensure(ErrorIndex { compiler }));
        let mut dylib_paths = builder.crablangc_lib_paths(compiler);
        dylib_paths.push(PathBuf::from(&builder.sysroot_libdir(compiler, compiler.host)));
        add_dylib_path(dylib_paths, &mut cmd);
        cmd
    }
}

impl Step for ErrorIndex {
    type Output = PathBuf;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/error_index_generator")
    }

    fn make_run(run: RunConfig<'_>) {
        // Compile the error-index in the same stage as crablangdoc to avoid
        // recompiling crablangdoc twice if we can.
        //
        // NOTE: This `make_run` isn't used in normal situations, only if you
        // manually build the tool with `x.py build
        // src/tools/error-index-generator` which almost nobody does.
        // Normally, `x.py test` or `x.py doc` will use the
        // `ErrorIndex::command` function instead.
        let compiler =
            run.builder.compiler(run.builder.top_stage.saturating_sub(1), run.builder.config.build);
        run.builder.ensure(ErrorIndex { compiler });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        builder
            .ensure(ToolBuild {
                compiler: self.compiler,
                target: self.compiler.host,
                tool: "error_index_generator",
                mode: Mode::ToolCrabLangc,
                path: "src/tools/error_index_generator",
                is_optional_tool: false,
                source_type: SourceType::InTree,
                extra_features: Vec::new(),
                allow_features: "",
            })
            .expect("expected to build -- essential tool")
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct RemoteTestServer {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for RemoteTestServer {
    type Output = PathBuf;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/remote-test-server")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(RemoteTestServer {
            compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        builder
            .ensure(ToolBuild {
                compiler: self.compiler,
                target: self.target,
                tool: "remote-test-server",
                mode: Mode::ToolStd,
                path: "src/tools/remote-test-server",
                is_optional_tool: false,
                source_type: SourceType::InTree,
                extra_features: Vec::new(),
                allow_features: "",
            })
            .expect("expected to build -- essential tool")
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct CrabLangdoc {
    /// This should only ever be 0 or 2.
    /// We sometimes want to reference the "bootstrap" crablangdoc, which is why this option is here.
    pub compiler: Compiler,
}

impl Step for CrabLangdoc {
    type Output = PathBuf;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/crablangdoc").path("src/libcrablangdoc")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangdoc {
            // Note: this is somewhat unique in that we actually want a *target*
            // compiler here, because crablangdoc *is* a compiler. We won't be using
            // this as the compiler to build with, but rather this is "what
            // compiler are we producing"?
            compiler: run.builder.compiler(run.builder.top_stage, run.target),
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        let target_compiler = self.compiler;
        if target_compiler.stage == 0 {
            if !target_compiler.is_snapshot(builder) {
                panic!("crablangdoc in stage 0 must be snapshot crablangdoc");
            }
            return builder.initial_crablangc.with_file_name(exe("crablangdoc", target_compiler.host));
        }
        let target = target_compiler.host;
        // Similar to `compile::Assemble`, build with the previous stage's compiler. Otherwise
        // we'd have stageN/bin/crablangc and stageN/bin/crablangdoc be effectively different stage
        // compilers, which isn't what we want. CrabLangdoc should be linked in the same way as the
        // crablangc compiler it's paired with, so it must be built with the previous stage compiler.
        let build_compiler = builder.compiler(target_compiler.stage - 1, builder.config.build);

        // When using `download-crablangc` and a stage0 build_compiler, copying crablangc doesn't actually
        // build stage0 libstd (because the libstd in sysroot has the wrong ABI). Explicitly build
        // it.
        builder.ensure(compile::Std::new(build_compiler, target_compiler.host));
        builder.ensure(compile::CrabLangc::new(build_compiler, target_compiler.host));
        // NOTE: this implies that `download-crablangc` is pretty useless when compiling with the stage0
        // compiler, since you do just as much work.
        if !builder.config.dry_run() && builder.download_crablangc() && build_compiler.stage == 0 {
            println!(
                "warning: `download-crablangc` does nothing when building stage1 tools; consider using `--stage 2` instead"
            );
        }

        // The presence of `target_compiler` ensures that the necessary libraries (codegen backends,
        // compiler libraries, ...) are built. CrabLangdoc does not require the presence of any
        // libraries within sysroot_libdir (i.e., crablanglib), though doctests may want it (since
        // they'll be linked to those libraries). As such, don't explicitly `ensure` any additional
        // libraries here. The intuition here is that If we've built a compiler, we should be able
        // to build crablangdoc.
        //
        let mut features = Vec::new();
        if builder.config.jemalloc {
            features.push("jemalloc".to_string());
        }

        let mut cargo = prepare_tool_cargo(
            builder,
            build_compiler,
            Mode::ToolCrabLangc,
            target,
            "build",
            "src/tools/crablangdoc",
            SourceType::InTree,
            features.as_slice(),
        );

        if builder.config.crablangc_parallel {
            cargo.crablangflag("--cfg=parallel_compiler");
        }

        let msg = tooling_output(
            Mode::ToolCrabLangc,
            "crablangdoc",
            build_compiler.stage,
            &self.compiler.host,
            &target,
        );
        builder.info(&msg);
        builder.run(&mut cargo.into());

        // Cargo adds a number of paths to the dylib search path on windows, which results in
        // the wrong crablangdoc being executed. To avoid the conflicting crablangdocs, we name the "tool"
        // crablangdoc a different name.
        let tool_crablangdoc = builder
            .cargo_out(build_compiler, Mode::ToolCrabLangc, target)
            .join(exe("crablangdoc_tool_binary", target_compiler.host));

        // don't create a stage0-sysroot/bin directory.
        if target_compiler.stage > 0 {
            let sysroot = builder.sysroot(target_compiler);
            let bindir = sysroot.join("bin");
            t!(fs::create_dir_all(&bindir));
            let bin_crablangdoc = bindir.join(exe("crablangdoc", target_compiler.host));
            let _ = fs::remove_file(&bin_crablangdoc);
            builder.copy(&tool_crablangdoc, &bin_crablangdoc);
            bin_crablangdoc
        } else {
            tool_crablangdoc
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Cargo {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for Cargo {
    type Output = PathBuf;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        run.path("src/tools/cargo").default_condition(
            builder.config.extended
                && builder.config.tools.as_ref().map_or(
                    true,
                    // If `tools` is set, search list for this tool.
                    |tools| tools.iter().any(|tool| tool == "cargo"),
                ),
        )
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Cargo {
            compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        let cargo_bin_path = builder
            .ensure(ToolBuild {
                compiler: self.compiler,
                target: self.target,
                tool: "cargo",
                mode: Mode::ToolCrabLangc,
                path: "src/tools/cargo",
                is_optional_tool: false,
                source_type: SourceType::Submodule,
                extra_features: Vec::new(),
                allow_features: "",
            })
            .expect("expected to build -- essential tool");

        let build_cred = |name, path| {
            // These credential helpers are currently experimental.
            // Any build failures will be ignored.
            let _ = builder.ensure(ToolBuild {
                compiler: self.compiler,
                target: self.target,
                tool: name,
                mode: Mode::ToolCrabLangc,
                path,
                is_optional_tool: true,
                source_type: SourceType::Submodule,
                extra_features: Vec::new(),
                allow_features: "",
            });
        };

        if self.target.contains("windows") {
            build_cred(
                "cargo-credential-wincred",
                "src/tools/cargo/crates/credential/cargo-credential-wincred",
            );
        }
        if self.target.contains("apple-darwin") {
            build_cred(
                "cargo-credential-macos-keychain",
                "src/tools/cargo/crates/credential/cargo-credential-macos-keychain",
            );
        }
        build_cred(
            "cargo-credential-1password",
            "src/tools/cargo/crates/credential/cargo-credential-1password",
        );
        cargo_bin_path
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct LldWrapper {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for LldWrapper {
    type Output = PathBuf;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        let src_exe = builder
            .ensure(ToolBuild {
                compiler: self.compiler,
                target: self.target,
                tool: "lld-wrapper",
                mode: Mode::ToolStd,
                path: "src/tools/lld-wrapper",
                is_optional_tool: false,
                source_type: SourceType::InTree,
                extra_features: Vec::new(),
                allow_features: "",
            })
            .expect("expected to build -- essential tool");

        src_exe
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangAnalyzer {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl CrabLangAnalyzer {
    pub const ALLOW_FEATURES: &str =
        "proc_macro_internals,proc_macro_diagnostic,proc_macro_span,proc_macro_span_shrink";
}

impl Step for CrabLangAnalyzer {
    type Output = Option<PathBuf>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        run.path("src/tools/crablang-analyzer").default_condition(
            builder.config.extended
                && builder
                    .config
                    .tools
                    .as_ref()
                    .map_or(true, |tools| tools.iter().any(|tool| tool == "crablang-analyzer")),
        )
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangAnalyzer {
            compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> Option<PathBuf> {
        builder.ensure(ToolBuild {
            compiler: self.compiler,
            target: self.target,
            tool: "crablang-analyzer",
            mode: Mode::ToolStd,
            path: "src/tools/crablang-analyzer",
            extra_features: vec!["crablang-analyzer/in-crablang-tree".to_owned()],
            is_optional_tool: false,
            source_type: SourceType::InTree,
            allow_features: CrabLangAnalyzer::ALLOW_FEATURES,
        })
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CrabLangAnalyzerProcMacroSrv {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CrabLangAnalyzerProcMacroSrv {
    type Output = Option<PathBuf>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        // Allow building `crablang-analyzer-proc-macro-srv` both as part of the `crablang-analyzer` and as a stand-alone tool.
        run.path("src/tools/crablang-analyzer")
            .path("src/tools/crablang-analyzer/crates/proc-macro-srv-cli")
            .default_condition(builder.config.tools.as_ref().map_or(true, |tools| {
                tools
                    .iter()
                    .any(|tool| tool == "crablang-analyzer" || tool == "crablang-analyzer-proc-macro-srv")
            }))
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CrabLangAnalyzerProcMacroSrv {
            compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> Option<PathBuf> {
        let path = builder.ensure(ToolBuild {
            compiler: self.compiler,
            target: self.target,
            tool: "crablang-analyzer-proc-macro-srv",
            mode: Mode::ToolStd,
            path: "src/tools/crablang-analyzer/crates/proc-macro-srv-cli",
            extra_features: vec!["proc-macro-srv/sysroot-abi".to_owned()],
            is_optional_tool: false,
            source_type: SourceType::InTree,
            allow_features: CrabLangAnalyzer::ALLOW_FEATURES,
        })?;

        // Copy `crablang-analyzer-proc-macro-srv` to `<sysroot>/libexec/`
        // so that r-a can use it.
        let libexec_path = builder.sysroot(self.compiler).join("libexec");
        t!(fs::create_dir_all(&libexec_path));
        builder.copy(&path, &libexec_path.join("crablang-analyzer-proc-macro-srv"));

        Some(path)
    }
}

macro_rules! tool_extended {
    (($sel:ident, $builder:ident),
       $($name:ident,
       $path:expr,
       $tool_name:expr,
       stable = $stable:expr
       $(,tool_std = $tool_std:literal)?
       $(,allow_features = $allow_features:expr)?
       ;)+) => {
        $(
            #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        pub struct $name {
            pub compiler: Compiler,
            pub target: TargetSelection,
            pub extra_features: Vec<String>,
        }

        impl Step for $name {
            type Output = Option<PathBuf>;
            const DEFAULT: bool = true; // Overwritten below
            const ONLY_HOSTS: bool = true;

            fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
                let builder = run.builder;
                run.path($path).default_condition(
                    builder.config.extended
                        && builder.config.tools.as_ref().map_or(
                            // By default, on nightly/dev enable all tools, else only
                            // build stable tools.
                            $stable || builder.build.unstable_features(),
                            // If `tools` is set, search list for this tool.
                            |tools| {
                                tools.iter().any(|tool| match tool.as_ref() {
                                    "clippy" => $tool_name == "clippy-driver",
                                    x => $tool_name == x,
                            })
                        }),
                )
            }

            fn make_run(run: RunConfig<'_>) {
                run.builder.ensure($name {
                    compiler: run.builder.compiler(run.builder.top_stage, run.builder.config.build),
                    target: run.target,
                    extra_features: Vec::new(),
                });
            }

            #[allow(unused_mut)]
            fn run(mut $sel, $builder: &Builder<'_>) -> Option<PathBuf> {
                $builder.ensure(ToolBuild {
                    compiler: $sel.compiler,
                    target: $sel.target,
                    tool: $tool_name,
                    mode: if false $(|| $tool_std)? { Mode::ToolStd } else { Mode::ToolCrabLangc },
                    path: $path,
                    extra_features: $sel.extra_features,
                    is_optional_tool: true,
                    source_type: SourceType::InTree,
                    allow_features: concat!($($allow_features)*),
                })
            }
        }
        )+
    }
}

// Note: tools need to be also added to `Builder::get_step_descriptions` in `builder.rs`
// to make `./x.py build <tool>` work.
// Note: Most submodule updates for tools are handled by bootstrap.py, since they're needed just to
// invoke Cargo to build bootstrap. See the comment there for more details.
tool_extended!((self, builder),
    Cargofmt, "src/tools/crablangfmt", "cargo-fmt", stable=true;
    CargoClippy, "src/tools/clippy", "cargo-clippy", stable=true;
    Clippy, "src/tools/clippy", "clippy-driver", stable=true;
    Miri, "src/tools/miri", "miri", stable=false;
    CargoMiri, "src/tools/miri/cargo-miri", "cargo-miri", stable=true;
    // FIXME: tool_std is not quite right, we shouldn't allow nightly features.
    // But `builder.cargo` doesn't know how to handle ToolBootstrap in stages other than 0,
    // and this is close enough for now.
    Rls, "src/tools/rls", "rls", stable=true, tool_std=true;
    CrabLangDemangler, "src/tools/crablang-demangler", "crablang-demangler", stable=false, tool_std=true;
    CrabLangfmt, "src/tools/crablangfmt", "crablangfmt", stable=true;
);

impl<'a> Builder<'a> {
    /// Gets a `Command` which is ready to run `tool` in `stage` built for
    /// `host`.
    pub fn tool_cmd(&self, tool: Tool) -> Command {
        let mut cmd = Command::new(self.tool_exe(tool));
        let compiler = self.compiler(0, self.config.build);
        let host = &compiler.host;
        // Prepares the `cmd` provided to be able to run the `compiler` provided.
        //
        // Notably this munges the dynamic library lookup path to point to the
        // right location to run `compiler`.
        let mut lib_paths: Vec<PathBuf> = vec![
            self.build.crablangc_snapshot_libdir(),
            self.cargo_out(compiler, Mode::ToolBootstrap, *host).join("deps"),
        ];

        // On MSVC a tool may invoke a C compiler (e.g., compiletest in run-make
        // mode) and that C compiler may need some extra PATH modification. Do
        // so here.
        if compiler.host.contains("msvc") {
            let curpaths = env::var_os("PATH").unwrap_or_default();
            let curpaths = env::split_paths(&curpaths).collect::<Vec<_>>();
            for &(ref k, ref v) in self.cc[&compiler.host].env() {
                if k != "PATH" {
                    continue;
                }
                for path in env::split_paths(v) {
                    if !curpaths.contains(&path) {
                        lib_paths.push(path);
                    }
                }
            }
        }

        add_dylib_path(lib_paths, &mut cmd);

        // Provide a CRABLANGC for this command to use.
        cmd.env("CRABLANGC", &self.initial_crablangc);

        cmd
    }
}
