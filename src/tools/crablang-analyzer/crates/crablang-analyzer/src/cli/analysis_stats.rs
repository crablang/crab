//! Fully type-check project and print various stats, like the number of type
//! errors.

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use hir::{
    db::{DefDatabase, ExpandDatabase, HirDatabase},
    AssocItem, Crate, Function, HasSource, HirDisplay, ModuleDef,
};
use hir_def::{
    body::{BodySourceMap, SyntheticSyntax},
    expr::{ExprId, PatId},
    FunctionId,
};
use hir_ty::{Interner, TyExt, TypeFlags};
use ide::{Analysis, AnalysisHost, LineCol, RootDatabase};
use ide_db::base_db::{
    salsa::{self, debug::DebugQueryTable, ParallelDatabase},
    SourceDatabase, SourceDatabaseExt,
};
use itertools::Itertools;
use oorandom::Rand32;
use profile::{Bytes, StopWatch};
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, CrabLangLibSource};
use rayon::prelude::*;
use crablangc_hash::FxHashSet;
use stdx::format_to;
use syntax::{AstNode, SyntaxNode};
use vfs::{AbsPathBuf, Vfs, VfsPath};

use crate::cli::{
    flags::{self, OutputFormat},
    load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice},
    print_memory_usage,
    progress_report::ProgressReport,
    report_metric, Result, Verbosity,
};

/// Need to wrap Snapshot to provide `Clone` impl for `map_with`
struct Snap<DB>(DB);
impl<DB: ParallelDatabase> Clone for Snap<salsa::Snapshot<DB>> {
    fn clone(&self) -> Snap<salsa::Snapshot<DB>> {
        Snap(self.0.snapshot())
    }
}

impl flags::AnalysisStats {
    pub fn run(self, verbosity: Verbosity) -> Result<()> {
        let mut rng = {
            let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            Rand32::new(seed)
        };

        let mut cargo_config = CargoConfig::default();
        cargo_config.sysroot = match self.no_sysroot {
            true => None,
            false => Some(CrabLangLibSource::Discover),
        };
        let no_progress = &|_| ();

        let mut db_load_sw = self.stop_watch();

        let path = AbsPathBuf::assert(env::current_dir()?.join(&self.path));
        let manifest = ProjectManifest::discover_single(&path)?;

        let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, no_progress)?;
        let metadata_time = db_load_sw.elapsed();
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: !self.disable_build_scripts,
            with_proc_macro_server: ProcMacroServerChoice::Sysroot,
            prefill_caches: false,
        };

        let build_scripts_time = if self.disable_build_scripts {
            None
        } else {
            let mut build_scripts_sw = self.stop_watch();
            let bs = workspace.run_build_scripts(&cargo_config, no_progress)?;
            workspace.set_build_scripts(bs);
            Some(build_scripts_sw.elapsed())
        };

        let (host, vfs, _proc_macro) =
            load_workspace(workspace, &cargo_config.extra_env, &load_cargo_config)?;
        let db = host.raw_database();
        eprint!("{:<20} {}", "Database loaded:", db_load_sw.elapsed());
        eprint!(" (metadata {metadata_time}");
        if let Some(build_scripts_time) = build_scripts_time {
            eprint!("; build {build_scripts_time}");
        }
        eprintln!(")");

        let mut analysis_sw = self.stop_watch();
        let mut num_crates = 0;
        let mut visited_modules = FxHashSet::default();
        let mut visit_queue = Vec::new();

        let mut krates = Crate::all(db);
        if self.randomize {
            shuffle(&mut rng, &mut krates);
        }
        for krate in krates {
            let module = krate.root_module(db);
            let file_id = module.definition_source(db).file_id;
            let file_id = file_id.original_file(db);
            let source_root = db.file_source_root(file_id);
            let source_root = db.source_root(source_root);
            if !source_root.is_library || self.with_deps {
                num_crates += 1;
                visit_queue.push(module);
            }
        }

        if self.randomize {
            shuffle(&mut rng, &mut visit_queue);
        }

        eprint!("  crates: {num_crates}");
        let mut num_decls = 0;
        let mut funcs = Vec::new();
        while let Some(module) = visit_queue.pop() {
            if visited_modules.insert(module) {
                visit_queue.extend(module.children(db));

                for decl in module.declarations(db) {
                    num_decls += 1;
                    if let ModuleDef::Function(f) = decl {
                        funcs.push(f);
                    }
                }

                for impl_def in module.impl_defs(db) {
                    for item in impl_def.items(db) {
                        num_decls += 1;
                        if let AssocItem::Function(f) = item {
                            funcs.push(f);
                        }
                    }
                }
            }
        }
        eprintln!(", mods: {}, decls: {num_decls}, fns: {}", visited_modules.len(), funcs.len());
        eprintln!("{:<20} {}", "Item Collection:", analysis_sw.elapsed());

        if self.randomize {
            shuffle(&mut rng, &mut funcs);
        }

        if !self.skip_inference {
            self.run_inference(&host, db, &vfs, &funcs, verbosity);
        }

        let total_span = analysis_sw.elapsed();
        eprintln!("{:<20} {total_span}", "Total:");
        report_metric("total time", total_span.time.as_millis() as u64, "ms");
        if let Some(instructions) = total_span.instructions {
            report_metric("total instructions", instructions, "#instr");
        }
        if let Some(memory) = total_span.memory {
            report_metric("total memory", memory.allocated.megabytes() as u64, "MB");
        }

        if env::var("RA_COUNT").is_ok() {
            eprintln!("{}", profile::countme::get_all());
        }

        if self.source_stats {
            let mut total_file_size = Bytes::default();
            for e in ide_db::base_db::ParseQuery.in_db(db).entries::<Vec<_>>() {
                total_file_size += syntax_len(db.parse(e.key).syntax_node())
            }

            let mut total_macro_file_size = Bytes::default();
            for e in hir::db::ParseMacroExpansionQuery.in_db(db).entries::<Vec<_>>() {
                if let Some((val, _)) = db.parse_macro_expansion(e.key).value {
                    total_macro_file_size += syntax_len(val.syntax_node())
                }
            }
            eprintln!("source files: {total_file_size}, macro files: {total_macro_file_size}");
        }

        if self.memory_usage && verbosity.is_verbose() {
            print_memory_usage(host, vfs);
        }

        Ok(())
    }

    fn run_inference(
        &self,
        host: &AnalysisHost,
        db: &RootDatabase,
        vfs: &Vfs,
        funcs: &[Function],
        verbosity: Verbosity,
    ) {
        let mut bar = match verbosity {
            Verbosity::Quiet | Verbosity::Spammy => ProgressReport::hidden(),
            _ if self.parallel || self.output.is_some() => ProgressReport::hidden(),
            _ => ProgressReport::new(funcs.len() as u64),
        };

        if self.parallel {
            let mut inference_sw = self.stop_watch();
            let snap = Snap(db.snapshot());
            funcs
                .par_iter()
                .map_with(snap, |snap, &f| {
                    let f_id = FunctionId::from(f);
                    snap.0.body(f_id.into());
                    snap.0.infer(f_id.into());
                })
                .count();
            eprintln!("{:<20} {}", "Parallel Inference:", inference_sw.elapsed());
        }

        let mut inference_sw = self.stop_watch();
        bar.tick();
        let mut num_exprs = 0;
        let mut num_exprs_unknown = 0;
        let mut num_exprs_partially_unknown = 0;
        let mut num_expr_type_mismatches = 0;
        let mut num_pats = 0;
        let mut num_pats_unknown = 0;
        let mut num_pats_partially_unknown = 0;
        let mut num_pat_type_mismatches = 0;
        let analysis = host.analysis();
        for f in funcs.iter().copied() {
            let name = f.name(db);
            let full_name = f
                .module(db)
                .path_to_root(db)
                .into_iter()
                .rev()
                .filter_map(|it| it.name(db))
                .chain(Some(f.name(db)))
                .join("::");
            if let Some(only_name) = self.only.as_deref() {
                if name.to_string() != only_name && full_name != only_name {
                    continue;
                }
            }
            let mut msg = format!("processing: {full_name}");
            if verbosity.is_verbose() {
                if let Some(src) = f.source(db) {
                    let original_file = src.file_id.original_file(db);
                    let path = vfs.file_path(original_file);
                    let syntax_range = src.value.syntax().text_range();
                    format_to!(msg, " ({} {:?})", path, syntax_range);
                }
            }
            if verbosity.is_spammy() {
                bar.println(msg.to_string());
            }
            bar.set_message(&msg);
            let f_id = FunctionId::from(f);
            let (body, sm) = db.body_with_source_map(f_id.into());
            let inference_result = db.infer(f_id.into());

            // region:expressions
            let (previous_exprs, previous_unknown, previous_partially_unknown) =
                (num_exprs, num_exprs_unknown, num_exprs_partially_unknown);
            for (expr_id, _) in body.exprs.iter() {
                let ty = &inference_result[expr_id];
                num_exprs += 1;
                let unknown_or_partial = if ty.is_unknown() {
                    num_exprs_unknown += 1;
                    if verbosity.is_spammy() {
                        if let Some((path, start, end)) =
                            expr_syntax_range(db, &analysis, vfs, &sm, expr_id)
                        {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Unknown type",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                            ));
                        } else {
                            bar.println(format!("{name}: Unknown type",));
                        }
                    }
                    true
                } else {
                    let is_partially_unknown =
                        ty.data(Interner).flags.contains(TypeFlags::HAS_ERROR);
                    if is_partially_unknown {
                        num_exprs_partially_unknown += 1;
                    }
                    is_partially_unknown
                };
                if self.only.is_some() && verbosity.is_spammy() {
                    // in super-verbose mode for just one function, we print every single expression
                    if let Some((_, start, end)) =
                        expr_syntax_range(db, &analysis, vfs, &sm, expr_id)
                    {
                        bar.println(format!(
                            "{}:{}-{}:{}: {}",
                            start.line + 1,
                            start.col,
                            end.line + 1,
                            end.col,
                            ty.display(db)
                        ));
                    } else {
                        bar.println(format!("unknown location: {}", ty.display(db)));
                    }
                }
                if unknown_or_partial && self.output == Some(OutputFormat::Csv) {
                    println!(
                        r#"{},type,"{}""#,
                        location_csv_expr(db, &analysis, vfs, &sm, expr_id),
                        ty.display(db)
                    );
                }
                if let Some(mismatch) = inference_result.type_mismatch_for_expr(expr_id) {
                    num_expr_type_mismatches += 1;
                    if verbosity.is_verbose() {
                        if let Some((path, start, end)) =
                            expr_syntax_range(db, &analysis, vfs, &sm, expr_id)
                        {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Expected {}, got {}",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        } else {
                            bar.println(format!(
                                "{}: Expected {}, got {}",
                                name,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        }
                    }
                    if self.output == Some(OutputFormat::Csv) {
                        println!(
                            r#"{},mismatch,"{}","{}""#,
                            location_csv_expr(db, &analysis, vfs, &sm, expr_id),
                            mismatch.expected.display(db),
                            mismatch.actual.display(db)
                        );
                    }
                }
            }
            if verbosity.is_spammy() {
                bar.println(format!(
                    "In {}: {} exprs, {} unknown, {} partial",
                    full_name,
                    num_exprs - previous_exprs,
                    num_exprs_unknown - previous_unknown,
                    num_exprs_partially_unknown - previous_partially_unknown
                ));
            }
            // endregion:expressions

            // region:patterns
            let (previous_pats, previous_unknown, previous_partially_unknown) =
                (num_pats, num_pats_unknown, num_pats_partially_unknown);
            for (pat_id, _) in body.pats.iter() {
                let ty = &inference_result[pat_id];
                num_pats += 1;
                let unknown_or_partial = if ty.is_unknown() {
                    num_pats_unknown += 1;
                    if verbosity.is_spammy() {
                        if let Some((path, start, end)) =
                            pat_syntax_range(db, &analysis, vfs, &sm, pat_id)
                        {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Unknown type",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                            ));
                        } else {
                            bar.println(format!("{name}: Unknown type",));
                        }
                    }
                    true
                } else {
                    let is_partially_unknown =
                        ty.data(Interner).flags.contains(TypeFlags::HAS_ERROR);
                    if is_partially_unknown {
                        num_pats_partially_unknown += 1;
                    }
                    is_partially_unknown
                };
                if self.only.is_some() && verbosity.is_spammy() {
                    // in super-verbose mode for just one function, we print every single pattern
                    if let Some((_, start, end)) = pat_syntax_range(db, &analysis, vfs, &sm, pat_id)
                    {
                        bar.println(format!(
                            "{}:{}-{}:{}: {}",
                            start.line + 1,
                            start.col,
                            end.line + 1,
                            end.col,
                            ty.display(db)
                        ));
                    } else {
                        bar.println(format!("unknown location: {}", ty.display(db)));
                    }
                }
                if unknown_or_partial && self.output == Some(OutputFormat::Csv) {
                    println!(
                        r#"{},type,"{}""#,
                        location_csv_pat(db, &analysis, vfs, &sm, pat_id),
                        ty.display(db)
                    );
                }
                if let Some(mismatch) = inference_result.type_mismatch_for_pat(pat_id) {
                    num_pat_type_mismatches += 1;
                    if verbosity.is_verbose() {
                        if let Some((path, start, end)) =
                            pat_syntax_range(db, &analysis, vfs, &sm, pat_id)
                        {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Expected {}, got {}",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        } else {
                            bar.println(format!(
                                "{}: Expected {}, got {}",
                                name,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        }
                    }
                    if self.output == Some(OutputFormat::Csv) {
                        println!(
                            r#"{},mismatch,"{}","{}""#,
                            location_csv_pat(db, &analysis, vfs, &sm, pat_id),
                            mismatch.expected.display(db),
                            mismatch.actual.display(db)
                        );
                    }
                }
            }
            if verbosity.is_spammy() {
                bar.println(format!(
                    "In {}: {} pats, {} unknown, {} partial",
                    full_name,
                    num_pats - previous_pats,
                    num_pats_unknown - previous_unknown,
                    num_pats_partially_unknown - previous_partially_unknown
                ));
            }
            // endregion:patterns
            bar.inc(1);
        }

        bar.finish_and_clear();
        eprintln!(
            "  exprs: {}, ??ty: {} ({}%), ?ty: {} ({}%), !ty: {}",
            num_exprs,
            num_exprs_unknown,
            percentage(num_exprs_unknown, num_exprs),
            num_exprs_partially_unknown,
            percentage(num_exprs_partially_unknown, num_exprs),
            num_expr_type_mismatches
        );
        eprintln!(
            "  pats: {}, ??ty: {} ({}%), ?ty: {} ({}%), !ty: {}",
            num_pats,
            num_pats_unknown,
            percentage(num_pats_unknown, num_pats),
            num_pats_partially_unknown,
            percentage(num_pats_partially_unknown, num_pats),
            num_pat_type_mismatches
        );
        report_metric("unknown type", num_exprs_unknown, "#");
        report_metric("type mismatches", num_expr_type_mismatches, "#");
        report_metric("pattern unknown type", num_pats_unknown, "#");
        report_metric("pattern type mismatches", num_pat_type_mismatches, "#");

        eprintln!("{:<20} {}", "Inference:", inference_sw.elapsed());
    }

    fn stop_watch(&self) -> StopWatch {
        StopWatch::start().memory(self.memory_usage)
    }
}

fn location_csv_expr(
    db: &RootDatabase,
    analysis: &Analysis,
    vfs: &Vfs,
    sm: &BodySourceMap,
    expr_id: ExprId,
) -> String {
    let src = match sm.expr_syntax(expr_id) {
        Ok(s) => s,
        Err(SyntheticSyntax) => return "synthetic,,".to_string(),
    };
    let root = db.parse_or_expand(src.file_id).unwrap();
    let node = src.map(|e| e.to_node(&root).syntax().clone());
    let original_range = node.as_ref().original_file_range(db);
    let path = vfs.file_path(original_range.file_id);
    let line_index = analysis.file_line_index(original_range.file_id).unwrap();
    let text_range = original_range.range;
    let (start, end) =
        (line_index.line_col(text_range.start()), line_index.line_col(text_range.end()));
    format!("{path},{}:{},{}:{}", start.line + 1, start.col, end.line + 1, end.col)
}

fn location_csv_pat(
    db: &RootDatabase,
    analysis: &Analysis,
    vfs: &Vfs,
    sm: &BodySourceMap,
    pat_id: PatId,
) -> String {
    let src = match sm.pat_syntax(pat_id) {
        Ok(s) => s,
        Err(SyntheticSyntax) => return "synthetic,,".to_string(),
    };
    let root = db.parse_or_expand(src.file_id).unwrap();
    let node = src.map(|e| {
        e.either(|it| it.to_node(&root).syntax().clone(), |it| it.to_node(&root).syntax().clone())
    });
    let original_range = node.as_ref().original_file_range(db);
    let path = vfs.file_path(original_range.file_id);
    let line_index = analysis.file_line_index(original_range.file_id).unwrap();
    let text_range = original_range.range;
    let (start, end) =
        (line_index.line_col(text_range.start()), line_index.line_col(text_range.end()));
    format!("{path},{}:{},{}:{}", start.line + 1, start.col, end.line + 1, end.col)
}

fn expr_syntax_range(
    db: &RootDatabase,
    analysis: &Analysis,
    vfs: &Vfs,
    sm: &BodySourceMap,
    expr_id: ExprId,
) -> Option<(VfsPath, LineCol, LineCol)> {
    let src = sm.expr_syntax(expr_id);
    if let Ok(src) = src {
        let root = db.parse_or_expand(src.file_id).unwrap();
        let node = src.map(|e| e.to_node(&root).syntax().clone());
        let original_range = node.as_ref().original_file_range(db);
        let path = vfs.file_path(original_range.file_id);
        let line_index = analysis.file_line_index(original_range.file_id).unwrap();
        let text_range = original_range.range;
        let (start, end) =
            (line_index.line_col(text_range.start()), line_index.line_col(text_range.end()));
        Some((path, start, end))
    } else {
        None
    }
}
fn pat_syntax_range(
    db: &RootDatabase,
    analysis: &Analysis,
    vfs: &Vfs,
    sm: &BodySourceMap,
    pat_id: PatId,
) -> Option<(VfsPath, LineCol, LineCol)> {
    let src = sm.pat_syntax(pat_id);
    if let Ok(src) = src {
        let root = db.parse_or_expand(src.file_id).unwrap();
        let node = src.map(|e| {
            e.either(
                |it| it.to_node(&root).syntax().clone(),
                |it| it.to_node(&root).syntax().clone(),
            )
        });
        let original_range = node.as_ref().original_file_range(db);
        let path = vfs.file_path(original_range.file_id);
        let line_index = analysis.file_line_index(original_range.file_id).unwrap();
        let text_range = original_range.range;
        let (start, end) =
            (line_index.line_col(text_range.start()), line_index.line_col(text_range.end()));
        Some((path, start, end))
    } else {
        None
    }
}

fn shuffle<T>(rng: &mut Rand32, slice: &mut [T]) {
    for i in 0..slice.len() {
        randomize_first(rng, &mut slice[i..]);
    }

    fn randomize_first<T>(rng: &mut Rand32, slice: &mut [T]) {
        assert!(!slice.is_empty());
        let idx = rng.rand_range(0..slice.len() as u32) as usize;
        slice.swap(0, idx);
    }
}

fn percentage(n: u64, total: u64) -> u64 {
    (n * 100).checked_div(total).unwrap_or(100)
}

fn syntax_len(node: SyntaxNode) -> usize {
    // Macro expanded code doesn't contain whitespace, so erase *all* whitespace
    // to make macro and non-macro code comparable.
    node.to_string().replace(|it: char| it.is_ascii_whitespace(), "").len()
}
