use crate::errors;
use crate::interface::{Compiler, Result};
use crate::proc_macro_decls;
use crate::util;

use crablangc_ast::{self as ast, visit};
use crablangc_borrowck as mir_borrowck;
use crablangc_codegen_ssa::traits::CodegenBackend;
use crablangc_data_structures::parallel;
use crablangc_data_structures::steal::Steal;
use crablangc_data_structures::sync::{Lrc, OnceCell, WorkerLocal};
use crablangc_errors::PResult;
use crablangc_expand::base::{ExtCtxt, LintStoreExpand};
use crablangc_fs_util::try_canonicalize;
use crablangc_hir::def_id::{StableCrateId, LOCAL_CRATE};
use crablangc_lint::{unerased_lint_store, BufferedEarlyLint, EarlyCheckNode, LintStore};
use crablangc_metadata::creader::CStore;
use crablangc_middle::arena::Arena;
use crablangc_middle::dep_graph::DepGraph;
use crablangc_middle::ty::query::{ExternProviders, Providers};
use crablangc_middle::ty::{self, GlobalCtxt, RegisteredTools, TyCtxt};
use crablangc_mir_build as mir_build;
use crablangc_parse::{parse_crate_from_file, parse_crate_from_source_str, validate_attr};
use crablangc_passes::{self, hir_stats, layout_test};
use crablangc_plugin_impl as plugin;
use crablangc_query_impl::{OnDiskCache, Queries as TcxQueries};
use crablangc_resolve::Resolver;
use crablangc_session::config::{CrateType, Input, OutputFilenames, OutputType};
use crablangc_session::cstore::{MetadataLoader, Untracked};
use crablangc_session::output::filename_for_input;
use crablangc_session::search_paths::PathKind;
use crablangc_session::{Limit, Session};
use crablangc_span::symbol::{sym, Symbol};
use crablangc_span::FileName;
use crablangc_target::spec::PanicStrategy;
use crablangc_trait_selection::traits;

use std::any::Any;
use std::ffi::OsString;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::{env, fs, iter};

pub fn parse<'a>(sess: &'a Session) -> PResult<'a, ast::Crate> {
    let krate = sess.time("parse_crate", || match &sess.io.input {
        Input::File(file) => parse_crate_from_file(file, &sess.parse_sess),
        Input::Str { input, name } => {
            parse_crate_from_source_str(name.clone(), input.clone(), &sess.parse_sess)
        }
    })?;

    if sess.opts.unstable_opts.input_stats {
        eprintln!("Lines of code:             {}", sess.source_map().count_lines());
        eprintln!("Pre-expansion node count:  {}", count_nodes(&krate));
    }

    if let Some(ref s) = sess.opts.unstable_opts.show_span {
        crablangc_ast_passes::show_span::run(sess.diagnostic(), s, &krate);
    }

    if sess.opts.unstable_opts.hir_stats {
        hir_stats::print_ast_stats(&krate, "PRE EXPANSION AST STATS", "ast-stats-1");
    }

    Ok(krate)
}

fn count_nodes(krate: &ast::Crate) -> usize {
    let mut counter = crablangc_ast_passes::node_count::NodeCounter::new();
    visit::walk_crate(&mut counter, krate);
    counter.count
}

pub fn register_plugins<'a>(
    sess: &'a Session,
    metadata_loader: &'a dyn MetadataLoader,
    register_lints: impl Fn(&Session, &mut LintStore),
    pre_configured_attrs: &[ast::Attribute],
    crate_name: Symbol,
) -> Result<LintStore> {
    // these need to be set "early" so that expansion sees `quote` if enabled.
    let features = crablangc_expand::config::features(sess, pre_configured_attrs);
    sess.init_features(features);

    let crate_types = util::collect_crate_types(sess, pre_configured_attrs);
    sess.init_crate_types(crate_types);

    let stable_crate_id = StableCrateId::new(
        crate_name,
        sess.crate_types().contains(&CrateType::Executable),
        sess.opts.cg.metadata.clone(),
    );
    sess.stable_crate_id.set(stable_crate_id).expect("not yet initialized");
    crablangc_incremental::prepare_session_directory(sess, crate_name, stable_crate_id)?;

    if sess.opts.incremental.is_some() {
        sess.time("incr_comp_garbage_collect_session_directories", || {
            if let Err(e) = crablangc_incremental::garbage_collect_session_directories(sess) {
                warn!(
                    "Error while trying to garbage collect incremental \
                     compilation cache directory: {}",
                    e
                );
            }
        });
    }

    let mut lint_store = crablangc_lint::new_lint_store(sess.enable_internal_lints());
    register_lints(sess, &mut lint_store);

    let registrars = sess.time("plugin_loading", || {
        plugin::load::load_plugins(sess, metadata_loader, pre_configured_attrs)
    });
    sess.time("plugin_registration", || {
        let mut registry = plugin::Registry { lint_store: &mut lint_store };
        for registrar in registrars {
            registrar(&mut registry);
        }
    });

    Ok(lint_store)
}

fn pre_expansion_lint<'a>(
    sess: &Session,
    lint_store: &LintStore,
    registered_tools: &RegisteredTools,
    check_node: impl EarlyCheckNode<'a>,
    node_name: Symbol,
) {
    sess.prof.generic_activity_with_arg("pre_AST_expansion_lint_checks", node_name.as_str()).run(
        || {
            crablangc_lint::check_ast_node(
                sess,
                true,
                lint_store,
                registered_tools,
                None,
                crablangc_lint::BuiltinCombinedPreExpansionLintPass::new(),
                check_node,
            );
        },
    );
}

// Cannot implement directly for `LintStore` due to trait coherence.
struct LintStoreExpandImpl<'a>(&'a LintStore);

impl LintStoreExpand for LintStoreExpandImpl<'_> {
    fn pre_expansion_lint(
        &self,
        sess: &Session,
        registered_tools: &RegisteredTools,
        node_id: ast::NodeId,
        attrs: &[ast::Attribute],
        items: &[crablangc_ast::ptr::P<ast::Item>],
        name: Symbol,
    ) {
        pre_expansion_lint(sess, self.0, registered_tools, (node_id, attrs, items), name);
    }
}

/// Runs the "early phases" of the compiler: initial `cfg` processing, loading compiler plugins,
/// syntax expansion, secondary `cfg` expansion, synthesis of a test
/// harness if one is to be provided, injection of a dependency on the
/// standard library and prelude, and name resolution.
#[instrument(level = "trace", skip(krate, resolver))]
fn configure_and_expand(
    mut krate: ast::Crate,
    pre_configured_attrs: &[ast::Attribute],
    resolver: &mut Resolver<'_, '_>,
) -> ast::Crate {
    let tcx = resolver.tcx();
    let sess = tcx.sess;
    let lint_store = unerased_lint_store(tcx);
    let crate_name = tcx.crate_name(LOCAL_CRATE);
    let lint_check_node = (&krate, pre_configured_attrs);
    pre_expansion_lint(sess, lint_store, tcx.registered_tools(()), lint_check_node, crate_name);
    crablangc_builtin_macros::register_builtin_macros(resolver);

    let num_standard_library_imports = sess.time("crate_injection", || {
        crablangc_builtin_macros::standard_library_imports::inject(
            &mut krate,
            pre_configured_attrs,
            resolver,
            sess,
        )
    });

    util::check_attr_crate_type(sess, pre_configured_attrs, &mut resolver.lint_buffer());

    // Expand all macros
    krate = sess.time("macro_expand_crate", || {
        // Windows dlls do not have rpaths, so they don't know how to find their
        // dependencies. It's up to us to tell the system where to find all the
        // dependent dlls. Note that this uses cfg!(windows) as opposed to
        // targ_cfg because syntax extensions are always loaded for the host
        // compiler, not for the target.
        //
        // This is somewhat of an inherently racy operation, however, as
        // multiple threads calling this function could possibly continue
        // extending PATH far beyond what it should. To solve this for now we
        // just don't add any new elements to PATH which are already there
        // within PATH. This is basically a targeted fix at #17360 for crablangdoc
        // which runs crablangc in parallel but has been seen (#33844) to cause
        // problems with PATH becoming too long.
        let mut old_path = OsString::new();
        if cfg!(windows) {
            old_path = env::var_os("PATH").unwrap_or(old_path);
            let mut new_path = sess.host_filesearch(PathKind::All).search_path_dirs();
            for path in env::split_paths(&old_path) {
                if !new_path.contains(&path) {
                    new_path.push(path);
                }
            }
            env::set_var(
                "PATH",
                &env::join_paths(
                    new_path.iter().filter(|p| env::join_paths(iter::once(p)).is_ok()),
                )
                .unwrap(),
            );
        }

        // Create the config for macro expansion
        let features = sess.features_untracked();
        let recursion_limit = get_recursion_limit(pre_configured_attrs, sess);
        let cfg = crablangc_expand::expand::ExpansionConfig {
            features: Some(features),
            recursion_limit,
            trace_mac: sess.opts.unstable_opts.trace_macros,
            should_test: sess.is_test_crate(),
            span_debug: sess.opts.unstable_opts.span_debug,
            proc_macro_backtrace: sess.opts.unstable_opts.proc_macro_backtrace,
            ..crablangc_expand::expand::ExpansionConfig::default(crate_name.to_string())
        };

        let lint_store = LintStoreExpandImpl(lint_store);
        let mut ecx = ExtCtxt::new(sess, cfg, resolver, Some(&lint_store));
        ecx.num_standard_library_imports = num_standard_library_imports;
        // Expand macros now!
        let krate = sess.time("expand_crate", || ecx.monotonic_expander().expand_crate(krate));

        // The rest is error reporting

        sess.parse_sess.buffered_lints.with_lock(|buffered_lints: &mut Vec<BufferedEarlyLint>| {
            buffered_lints.append(&mut ecx.buffered_early_lint);
        });

        sess.time("check_unused_macros", || {
            ecx.check_unused_macros();
        });

        // If we hit a recursion limit, exit early to avoid later passes getting overwhelmed
        // with a large AST
        if ecx.reduced_recursion_limit.is_some() {
            sess.abort_if_errors();
            unreachable!();
        }

        if cfg!(windows) {
            env::set_var("PATH", &old_path);
        }

        krate
    });

    sess.time("maybe_building_test_harness", || {
        crablangc_builtin_macros::test_harness::inject(&mut krate, sess, resolver)
    });

    let has_proc_macro_decls = sess.time("AST_validation", || {
        crablangc_ast_passes::ast_validation::check_crate(sess, &krate, resolver.lint_buffer())
    });

    let crate_types = sess.crate_types();
    let is_executable_crate = crate_types.contains(&CrateType::Executable);
    let is_proc_macro_crate = crate_types.contains(&CrateType::ProcMacro);

    if crate_types.len() > 1 {
        if is_executable_crate {
            sess.emit_err(errors::MixedBinCrate);
        }
        if is_proc_macro_crate {
            sess.emit_err(errors::MixedProcMacroCrate);
        }
    }

    if is_proc_macro_crate && sess.panic_strategy() == PanicStrategy::Abort {
        sess.emit_warning(errors::ProcMacroCratePanicAbort);
    }

    sess.time("maybe_create_a_macro_crate", || {
        let is_test_crate = sess.is_test_crate();
        crablangc_builtin_macros::proc_macro_harness::inject(
            &mut krate,
            sess,
            resolver,
            is_proc_macro_crate,
            has_proc_macro_decls,
            is_test_crate,
            sess.diagnostic(),
        )
    });

    // Done with macro expansion!

    resolver.resolve_crate(&krate);

    krate
}

fn early_lint_checks(tcx: TyCtxt<'_>, (): ()) {
    let sess = tcx.sess;
    let (resolver, krate) = &*tcx.resolver_for_lowering(()).borrow();
    let mut lint_buffer = resolver.lint_buffer.steal();

    if sess.opts.unstable_opts.input_stats {
        eprintln!("Post-expansion node count: {}", count_nodes(&krate));
    }

    if sess.opts.unstable_opts.hir_stats {
        hir_stats::print_ast_stats(&krate, "POST EXPANSION AST STATS", "ast-stats-2");
    }

    // Needs to go *after* expansion to be able to check the results of macro expansion.
    sess.time("complete_gated_feature_checking", || {
        crablangc_ast_passes::feature_gate::check_crate(&krate, sess);
    });

    // Add all buffered lints from the `ParseSess` to the `Session`.
    sess.parse_sess.buffered_lints.with_lock(|buffered_lints| {
        info!("{} parse sess buffered_lints", buffered_lints.len());
        for early_lint in buffered_lints.drain(..) {
            lint_buffer.add_early_lint(early_lint);
        }
    });

    // Gate identifiers containing invalid Unicode codepoints that were recovered during lexing.
    sess.parse_sess.bad_unicode_identifiers.with_lock(|identifiers| {
        let mut identifiers: Vec<_> = identifiers.drain().collect();
        identifiers.sort_by_key(|&(key, _)| key);
        for (ident, mut spans) in identifiers.into_iter() {
            spans.sort();
            if ident == sym::ferris {
                let first_span = spans[0];
                sess.emit_err(errors::FerrisIdentifier { spans, first_span });
            } else {
                sess.emit_err(errors::EmojiIdentifier { spans, ident });
            }
        }
    });

    let lint_store = unerased_lint_store(tcx);
    crablangc_lint::check_ast_node(
        sess,
        false,
        lint_store,
        tcx.registered_tools(()),
        Some(lint_buffer),
        crablangc_lint::BuiltinCombinedEarlyLintPass::new(),
        (&**krate, &*krate.attrs),
    )
}

// Returns all the paths that correspond to generated files.
fn generated_output_paths(
    sess: &Session,
    outputs: &OutputFilenames,
    exact_name: bool,
    crate_name: Symbol,
) -> Vec<PathBuf> {
    let mut out_filenames = Vec::new();
    for output_type in sess.opts.output_types.keys() {
        let file = outputs.path(*output_type);
        match *output_type {
            // If the filename has been overridden using `-o`, it will not be modified
            // by appending `.rlib`, `.exe`, etc., so we can skip this transformation.
            OutputType::Exe if !exact_name => {
                for crate_type in sess.crate_types().iter() {
                    let p = filename_for_input(sess, *crate_type, crate_name, outputs);
                    out_filenames.push(p);
                }
            }
            OutputType::DepInfo if sess.opts.unstable_opts.dep_info_omit_d_target => {
                // Don't add the dep-info output when omitting it from dep-info targets
            }
            _ => {
                out_filenames.push(file);
            }
        }
    }
    out_filenames
}

// Runs `f` on every output file path and returns the first non-None result, or None if `f`
// returns None for every file path.
fn check_output<F, T>(output_paths: &[PathBuf], f: F) -> Option<T>
where
    F: Fn(&PathBuf) -> Option<T>,
{
    for output_path in output_paths {
        if let Some(result) = f(output_path) {
            return Some(result);
        }
    }
    None
}

fn output_contains_path(output_paths: &[PathBuf], input_path: &Path) -> bool {
    let input_path = try_canonicalize(input_path).ok();
    if input_path.is_none() {
        return false;
    }
    let check = |output_path: &PathBuf| {
        if try_canonicalize(output_path).ok() == input_path { Some(()) } else { None }
    };
    check_output(output_paths, check).is_some()
}

fn output_conflicts_with_dir(output_paths: &[PathBuf]) -> Option<PathBuf> {
    let check = |output_path: &PathBuf| output_path.is_dir().then(|| output_path.clone());
    check_output(output_paths, check)
}

fn escape_dep_filename(filename: &str) -> String {
    // Apparently clang and gcc *only* escape spaces:
    // https://llvm.org/klaus/clang/commit/9d50634cfc268ecc9a7250226dd5ca0e945240d4
    filename.replace(' ', "\\ ")
}

// Makefile comments only need escaping newlines and `\`.
// The result can be unescaped by anything that can unescape `escape_default` and friends.
fn escape_dep_env(symbol: Symbol) -> String {
    let s = symbol.as_str();
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\n' => escaped.push_str(r"\n"),
            '\r' => escaped.push_str(r"\r"),
            '\\' => escaped.push_str(r"\\"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn write_out_deps(tcx: TyCtxt<'_>, outputs: &OutputFilenames, out_filenames: &[PathBuf]) {
    // Write out dependency rules to the dep-info file if requested
    let sess = tcx.sess;
    if !sess.opts.output_types.contains_key(&OutputType::DepInfo) {
        return;
    }
    let deps_filename = outputs.path(OutputType::DepInfo);

    let result: io::Result<()> = try {
        // Build a list of files used to compile the output and
        // write Makefile-compatible dependency rules
        let mut files: Vec<String> = sess
            .source_map()
            .files()
            .iter()
            .filter(|fmap| fmap.is_real_file())
            .filter(|fmap| !fmap.is_imported())
            .map(|fmap| escape_dep_filename(&fmap.name.prefer_local().to_string()))
            .collect();

        // Account for explicitly marked-to-track files
        // (e.g. accessed in proc macros).
        let file_depinfo = sess.parse_sess.file_depinfo.borrow();

        let normalize_path = |path: PathBuf| {
            let file = FileName::from(path);
            escape_dep_filename(&file.prefer_local().to_string())
        };

        let extra_tracked_files =
            file_depinfo.iter().map(|path_sym| normalize_path(PathBuf::from(path_sym.as_str())));
        files.extend(extra_tracked_files);

        // We also need to track used PGO profile files
        if let Some(ref profile_instr) = sess.opts.cg.profile_use {
            files.push(normalize_path(profile_instr.as_path().to_path_buf()));
        }
        if let Some(ref profile_sample) = sess.opts.unstable_opts.profile_sample_use {
            files.push(normalize_path(profile_sample.as_path().to_path_buf()));
        }

        if sess.binary_dep_depinfo() {
            if let Some(ref backend) = sess.opts.unstable_opts.codegen_backend {
                if backend.contains('.') {
                    // If the backend name contain a `.`, it is the path to an external dynamic
                    // library. If not, it is not a path.
                    files.push(backend.to_string());
                }
            }

            for &cnum in tcx.crates(()) {
                let source = tcx.used_crate_source(cnum);
                if let Some((path, _)) = &source.dylib {
                    files.push(escape_dep_filename(&path.display().to_string()));
                }
                if let Some((path, _)) = &source.rlib {
                    files.push(escape_dep_filename(&path.display().to_string()));
                }
                if let Some((path, _)) = &source.rmeta {
                    files.push(escape_dep_filename(&path.display().to_string()));
                }
            }
        }

        let mut file = BufWriter::new(fs::File::create(&deps_filename)?);
        for path in out_filenames {
            writeln!(file, "{}: {}\n", path.display(), files.join(" "))?;
        }

        // Emit a fake target for each input file to the compilation. This
        // prevents `make` from spitting out an error if a file is later
        // deleted. For more info see #28735
        for path in files {
            writeln!(file, "{path}:")?;
        }

        // Emit special comments with information about accessed environment variables.
        let env_depinfo = sess.parse_sess.env_depinfo.borrow();
        if !env_depinfo.is_empty() {
            let mut envs: Vec<_> = env_depinfo
                .iter()
                .map(|(k, v)| (escape_dep_env(*k), v.map(escape_dep_env)))
                .collect();
            envs.sort_unstable();
            writeln!(file)?;
            for (k, v) in envs {
                write!(file, "# env-dep:{k}")?;
                if let Some(v) = v {
                    write!(file, "={v}")?;
                }
                writeln!(file)?;
            }
        }
    };

    match result {
        Ok(_) => {
            if sess.opts.json_artifact_notifications {
                sess.parse_sess
                    .span_diagnostic
                    .emit_artifact_notification(&deps_filename, "dep-info");
            }
        }
        Err(error) => {
            sess.emit_fatal(errors::ErrorWritingDependencies { path: &deps_filename, error });
        }
    }
}

fn resolver_for_lowering<'tcx>(
    tcx: TyCtxt<'tcx>,
    (): (),
) -> &'tcx Steal<(ty::ResolverAstLowering, Lrc<ast::Crate>)> {
    let arenas = Resolver::arenas();
    let _ = tcx.registered_tools(()); // Uses `crate_for_resolver`.
    let (krate, pre_configured_attrs) = tcx.crate_for_resolver(()).steal();
    let mut resolver = Resolver::new(tcx, &pre_configured_attrs, krate.spans.inner_span, &arenas);
    let krate = configure_and_expand(krate, &pre_configured_attrs, &mut resolver);

    // Make sure we don't mutate the cstore from here on.
    tcx.untracked().cstore.leak();

    let ty::ResolverOutputs {
        global_ctxt: untracked_resolutions,
        ast_lowering: untracked_resolver_for_lowering,
    } = resolver.into_outputs();

    let feed = tcx.feed_unit_query();
    feed.resolutions(tcx.arena.alloc(untracked_resolutions));
    tcx.arena.alloc(Steal::new((untracked_resolver_for_lowering, Lrc::new(krate))))
}

fn output_filenames(tcx: TyCtxt<'_>, (): ()) -> Arc<OutputFilenames> {
    let sess = tcx.sess;
    let _timer = sess.timer("prepare_outputs");
    let (_, krate) = &*tcx.resolver_for_lowering(()).borrow();
    let crate_name = tcx.crate_name(LOCAL_CRATE);

    // FIXME: crablangdoc passes &[] instead of &krate.attrs here
    let outputs = util::build_output_filenames(&krate.attrs, sess);

    let output_paths =
        generated_output_paths(sess, &outputs, sess.io.output_file.is_some(), crate_name);

    // Ensure the source file isn't accidentally overwritten during compilation.
    if let Some(ref input_path) = sess.io.input.opt_path() {
        if sess.opts.will_create_output_file() {
            if output_contains_path(&output_paths, input_path) {
                sess.emit_fatal(errors::InputFileWouldBeOverWritten { path: input_path });
            }
            if let Some(ref dir_path) = output_conflicts_with_dir(&output_paths) {
                sess.emit_fatal(errors::GeneratedFileConflictsWithDirectory {
                    input_path,
                    dir_path,
                });
            }
        }
    }

    if let Some(ref dir) = sess.io.temps_dir {
        if fs::create_dir_all(dir).is_err() {
            sess.emit_fatal(errors::TempsDirError);
        }
    }

    write_out_deps(tcx, &outputs, &output_paths);

    let only_dep_info = sess.opts.output_types.contains_key(&OutputType::DepInfo)
        && sess.opts.output_types.len() == 1;

    if !only_dep_info {
        if let Some(ref dir) = sess.io.output_dir {
            if fs::create_dir_all(dir).is_err() {
                sess.emit_fatal(errors::OutDirError);
            }
        }
    }

    outputs.into()
}

pub static DEFAULT_QUERY_PROVIDERS: LazyLock<Providers> = LazyLock::new(|| {
    let providers = &mut Providers::default();
    providers.analysis = analysis;
    providers.hir_crate = crablangc_ast_lowering::lower_to_hir;
    providers.output_filenames = output_filenames;
    providers.resolver_for_lowering = resolver_for_lowering;
    providers.early_lint_checks = early_lint_checks;
    proc_macro_decls::provide(providers);
    crablangc_const_eval::provide(providers);
    crablangc_middle::hir::provide(providers);
    mir_borrowck::provide(providers);
    mir_build::provide(providers);
    crablangc_mir_transform::provide(providers);
    crablangc_monomorphize::provide(providers);
    crablangc_privacy::provide(providers);
    crablangc_resolve::provide(providers);
    crablangc_hir_analysis::provide(providers);
    crablangc_hir_typeck::provide(providers);
    ty::provide(providers);
    traits::provide(providers);
    crablangc_passes::provide(providers);
    crablangc_traits::provide(providers);
    crablangc_ty_utils::provide(providers);
    crablangc_metadata::provide(providers);
    crablangc_lint::provide(providers);
    crablangc_symbol_mangling::provide(providers);
    crablangc_codegen_ssa::provide(providers);
    *providers
});

pub static DEFAULT_EXTERN_QUERY_PROVIDERS: LazyLock<ExternProviders> = LazyLock::new(|| {
    let mut extern_providers = ExternProviders::default();
    crablangc_metadata::provide_extern(&mut extern_providers);
    crablangc_codegen_ssa::provide_extern(&mut extern_providers);
    extern_providers
});

pub fn create_global_ctxt<'tcx>(
    compiler: &'tcx Compiler,
    lint_store: Lrc<LintStore>,
    dep_graph: DepGraph,
    untracked: Untracked,
    queries: &'tcx OnceCell<TcxQueries<'tcx>>,
    gcx_cell: &'tcx OnceCell<GlobalCtxt<'tcx>>,
    arena: &'tcx WorkerLocal<Arena<'tcx>>,
    hir_arena: &'tcx WorkerLocal<crablangc_hir::Arena<'tcx>>,
) -> &'tcx GlobalCtxt<'tcx> {
    // We're constructing the HIR here; we don't care what we will
    // read, since we haven't even constructed the *input* to
    // incr. comp. yet.
    dep_graph.assert_ignored();

    let sess = &compiler.session();
    let query_result_on_disk_cache = crablangc_incremental::load_query_result_cache(sess);

    let codegen_backend = compiler.codegen_backend();
    let mut local_providers = *DEFAULT_QUERY_PROVIDERS;
    codegen_backend.provide(&mut local_providers);

    let mut extern_providers = *DEFAULT_EXTERN_QUERY_PROVIDERS;
    codegen_backend.provide_extern(&mut extern_providers);

    if let Some(callback) = compiler.override_queries {
        callback(sess, &mut local_providers, &mut extern_providers);
    }

    let queries = queries.get_or_init(|| {
        TcxQueries::new(local_providers, extern_providers, query_result_on_disk_cache)
    });

    sess.time("setup_global_ctxt", || {
        gcx_cell.get_or_init(move || {
            TyCtxt::create_global_ctxt(
                sess,
                lint_store,
                arena,
                hir_arena,
                untracked,
                dep_graph,
                queries.on_disk_cache.as_ref().map(OnDiskCache::as_dyn),
                queries.as_dyn(),
                crablangc_query_impl::query_callbacks(arena),
            )
        })
    })
}

/// Runs the resolution, type-checking, region checking and other
/// miscellaneous analysis passes on the crate.
fn analysis(tcx: TyCtxt<'_>, (): ()) -> Result<()> {
    crablangc_passes::hir_id_validator::check_crate(tcx);

    let sess = tcx.sess;
    let mut entry_point = None;

    sess.time("misc_checking_1", || {
        parallel!(
            {
                entry_point = sess.time("looking_for_entry_point", || tcx.entry_fn(()));

                sess.time("looking_for_derive_registrar", || {
                    tcx.ensure().proc_macro_decls_static(())
                });

                CStore::from_tcx(tcx).report_unused_deps(tcx);
            },
            {
                tcx.hir().par_for_each_module(|module| {
                    tcx.ensure().check_mod_loops(module);
                    tcx.ensure().check_mod_attrs(module);
                    tcx.ensure().check_mod_naked_functions(module);
                    tcx.ensure().check_mod_unstable_api_usage(module);
                    tcx.ensure().check_mod_const_bodies(module);
                });
            },
            {
                sess.time("unused_lib_feature_checking", || {
                    crablangc_passes::stability::check_unused_or_stable_features(tcx)
                });
            },
            {
                // We force these queries to run,
                // since they might not otherwise get called.
                // This marks the corresponding crate-level attributes
                // as used, and ensures that their values are valid.
                tcx.ensure().limits(());
                tcx.ensure().stability_index(());
            }
        );
    });

    // passes are timed inside typeck
    crablangc_hir_analysis::check_crate(tcx)?;

    sess.time("misc_checking_2", || {
        parallel!(
            {
                sess.time("match_checking", || {
                    tcx.hir().par_body_owners(|def_id| tcx.ensure().check_match(def_id))
                });
            },
            {
                sess.time("liveness_checking", || {
                    tcx.hir().par_body_owners(|def_id| {
                        // this must run before MIR dump, because
                        // "not all control paths return a value" is reported here.
                        //
                        // maybe move the check to a MIR pass?
                        tcx.ensure().check_liveness(def_id.to_def_id());
                    });
                });
            }
        );
    });

    sess.time("MIR_borrow_checking", || {
        tcx.hir().par_body_owners(|def_id| tcx.ensure().mir_borrowck(def_id));
    });

    sess.time("MIR_effect_checking", || {
        for def_id in tcx.hir().body_owners() {
            tcx.ensure().thir_check_unsafety(def_id);
            if !tcx.sess.opts.unstable_opts.thir_unsafeck {
                crablangc_mir_transform::check_unsafety::check_unsafety(tcx, def_id);
            }
            tcx.ensure().has_ffi_unwind_calls(def_id);

            if tcx.hir().body_const_context(def_id).is_some() {
                tcx.ensure()
                    .mir_drops_elaborated_and_const_checked(ty::WithOptConstParam::unknown(def_id));
            }
        }
    });

    if tcx.sess.opts.unstable_opts.drop_tracking_mir {
        tcx.hir().par_body_owners(|def_id| {
            if let crablangc_hir::def::DefKind::Generator = tcx.def_kind(def_id) {
                tcx.ensure().mir_generator_witnesses(def_id);
                tcx.ensure().check_generator_obligations(def_id);
            }
        });
    }

    sess.time("layout_testing", || layout_test::test_layout(tcx));

    // Avoid overwhelming user with errors if borrow checking failed.
    // I'm not sure how helpful this is, to be honest, but it avoids a
    // lot of annoying errors in the ui tests (basically,
    // lint warnings and so on -- kindck used to do this abort, but
    // kindck is gone now). -nmatsakis
    if let Some(reported) = sess.has_errors() {
        return Err(reported);
    }

    sess.time("misc_checking_3", || {
        parallel!(
            {
                tcx.ensure().effective_visibilities(());

                parallel!(
                    {
                        tcx.ensure().check_private_in_public(());
                    },
                    {
                        tcx.hir()
                            .par_for_each_module(|module| tcx.ensure().check_mod_deathness(module));
                    },
                    {
                        sess.time("lint_checking", || {
                            crablangc_lint::check_crate(tcx, || {
                                crablangc_lint::BuiltinCombinedLateLintPass::new()
                            });
                        });
                    }
                );
            },
            {
                sess.time("privacy_checking_modules", || {
                    tcx.hir().par_for_each_module(|module| {
                        tcx.ensure().check_mod_privacy(module);
                    });
                });
            }
        );

        // This check has to be run after all lints are done processing. We don't
        // define a lint filter, as all lint checks should have finished at this point.
        sess.time("check_lint_expectations", || tcx.check_expectations(None));
    });

    Ok(())
}

/// Runs the codegen backend, after which the AST and analysis can
/// be discarded.
pub fn start_codegen<'tcx>(
    codegen_backend: &dyn CodegenBackend,
    tcx: TyCtxt<'tcx>,
) -> Box<dyn Any> {
    info!("Pre-codegen\n{:?}", tcx.debug_stats());

    let (metadata, need_metadata_module) = crablangc_metadata::fs::encode_and_write_metadata(tcx);

    let codegen = tcx.sess.time("codegen_crate", move || {
        codegen_backend.codegen_crate(tcx, metadata, need_metadata_module)
    });

    // Don't run these test assertions when not doing codegen. Compiletest tries to build
    // build-fail tests in check mode first and expects it to not give an error in that case.
    if tcx.sess.opts.output_types.should_codegen() {
        crablangc_incremental::assert_module_sources::assert_module_sources(tcx);
        crablangc_symbol_mangling::test::report_symbol_names(tcx);
    }

    info!("Post-codegen\n{:?}", tcx.debug_stats());

    if tcx.sess.opts.output_types.contains_key(&OutputType::Mir) {
        if let Err(error) = crablangc_mir_transform::dump_mir::emit_mir(tcx) {
            tcx.sess.emit_err(errors::CantEmitMIR { error });
            tcx.sess.abort_if_errors();
        }
    }

    codegen
}

fn get_recursion_limit(krate_attrs: &[ast::Attribute], sess: &Session) -> Limit {
    if let Some(attr) = krate_attrs
        .iter()
        .find(|attr| attr.has_name(sym::recursion_limit) && attr.value_str().is_none())
    {
        // This is here mainly to check for using a macro, such as
        // #![recursion_limit = foo!()]. That is not supported since that
        // would require expanding this while in the middle of expansion,
        // which needs to know the limit before expanding. Otherwise,
        // validation would normally be caught in AstValidator (via
        // `check_builtin_attribute`), but by the time that runs the macro
        // is expanded, and it doesn't give an error.
        validate_attr::emit_fatal_malformed_builtin_attribute(
            &sess.parse_sess,
            attr,
            sym::recursion_limit,
        );
    }
    crablangc_middle::middle::limits::get_recursion_limit(krate_attrs, sess)
}
