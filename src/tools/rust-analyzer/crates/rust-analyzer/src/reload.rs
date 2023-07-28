//! Project loading & configuration updates.
//!
//! This is quite tricky. The main problem is time and changes -- there's no
//! fixed "project" rust-analyzer is working with, "current project" is itself
//! mutable state. For example, when the user edits `Cargo.toml` by adding a new
//! dependency, project model changes. What's more, switching project model is
//! not instantaneous -- it takes time to run `cargo metadata` and (for proc
//! macros) `cargo check`.
//!
//! The main guiding principle here is, as elsewhere in rust-analyzer,
//! robustness. We try not to assume that the project model exists or is
//! correct. Instead, we try to provide a best-effort service. Even if the
//! project is currently loading and we don't have a full project model, we
//! still want to respond to various  requests.
use std::{iter, mem};

use flycheck::{FlycheckConfig, FlycheckHandle};
use hir::db::DefDatabase;
use ide::Change;
use ide_db::{
    base_db::{salsa::Durability, CrateGraph, ProcMacroPaths, ProcMacros},
    FxHashMap,
};
use load_cargo::{load_proc_macro, ProjectFolders};
use proc_macro_api::ProcMacroServer;
use project_model::{ProjectWorkspace, WorkspaceBuildScripts};
use rustc_hash::FxHashSet;
use stdx::{format_to, thread::ThreadIntent};
use triomphe::Arc;
use vfs::{AbsPath, ChangeKind};

use crate::{
    config::{Config, FilesWatcher, LinkedProject},
    global_state::GlobalState,
    lsp_ext,
    main_loop::Task,
    op_queue::Cause,
};

#[derive(Debug)]
pub(crate) enum ProjectWorkspaceProgress {
    Begin,
    Report(String),
    End(Vec<anyhow::Result<ProjectWorkspace>>, bool),
}

#[derive(Debug)]
pub(crate) enum BuildDataProgress {
    Begin,
    Report(String),
    End((Arc<Vec<ProjectWorkspace>>, Vec<anyhow::Result<WorkspaceBuildScripts>>)),
}

#[derive(Debug)]
pub(crate) enum ProcMacroProgress {
    Begin,
    Report(String),
    End(ProcMacros),
}

impl GlobalState {
    pub(crate) fn is_quiescent(&self) -> bool {
        !(self.last_reported_status.is_none()
            || self.fetch_workspaces_queue.op_in_progress()
            || self.fetch_build_data_queue.op_in_progress()
            || self.fetch_proc_macros_queue.op_in_progress()
            || self.vfs_progress_config_version < self.vfs_config_version
            || self.vfs_progress_n_done < self.vfs_progress_n_total)
    }

    pub(crate) fn update_configuration(&mut self, config: Config) {
        let _p = profile::span("GlobalState::update_configuration");
        let old_config = mem::replace(&mut self.config, Arc::new(config));
        if self.config.lru_parse_query_capacity() != old_config.lru_parse_query_capacity() {
            self.analysis_host.update_lru_capacity(self.config.lru_parse_query_capacity());
        }
        if self.config.lru_query_capacities() != old_config.lru_query_capacities() {
            self.analysis_host.update_lru_capacities(
                &self.config.lru_query_capacities().cloned().unwrap_or_default(),
            );
        }
        if self.config.linked_projects() != old_config.linked_projects() {
            self.fetch_workspaces_queue.request_op("linked projects changed".to_string(), false)
        } else if self.config.flycheck() != old_config.flycheck() {
            self.reload_flycheck();
        }

        if self.analysis_host.raw_database().expand_proc_attr_macros()
            != self.config.expand_proc_attr_macros()
        {
            self.analysis_host.raw_database_mut().set_expand_proc_attr_macros_with_durability(
                self.config.expand_proc_attr_macros(),
                Durability::HIGH,
            );
        }
    }

    pub(crate) fn current_status(&self) -> lsp_ext::ServerStatusParams {
        let mut status = lsp_ext::ServerStatusParams {
            health: lsp_ext::Health::Ok,
            quiescent: self.is_quiescent(),
            message: None,
        };
        let mut message = String::new();

        if self.proc_macro_changed {
            status.health = lsp_ext::Health::Warning;
            message.push_str("Proc-macros have changed and need to be rebuilt.\n\n");
        }
        if let Err(_) = self.fetch_build_data_error() {
            status.health = lsp_ext::Health::Warning;
            message.push_str("Failed to run build scripts of some packages.\n\n");
        }
        if self.proc_macro_clients.iter().any(|it| it.is_err()) {
            status.health = lsp_ext::Health::Warning;
            message.push_str("Failed to spawn one or more proc-macro servers.\n\n");
        }
        if !self.config.cargo_autoreload()
            && self.is_quiescent()
            && self.fetch_workspaces_queue.op_requested()
        {
            status.health = lsp_ext::Health::Warning;
            message.push_str("Auto-reloading is disabled and the workspace has changed, a manual workspace reload is required.\n\n");
        }
        if self.config.linked_projects().is_empty()
            && self.config.detached_files().is_empty()
            && self.config.notifications().cargo_toml_not_found
        {
            status.health = lsp_ext::Health::Warning;
            message.push_str("Failed to discover workspace.\n");
            message.push_str("Consider adding the `Cargo.toml` of the workspace to the [`linkedProjects`](https://rust-analyzer.github.io/manual.html#rust-analyzer.linkedProjects) setting.\n\n");
        }
        if let Some(err) = &self.config_errors {
            status.health = lsp_ext::Health::Warning;
            format_to!(message, "{err}\n");
        }
        if let Some(err) = &self.last_flycheck_error {
            status.health = lsp_ext::Health::Warning;
            message.push_str(err);
            message.push('\n');
        }

        for ws in self.workspaces.iter() {
            let (ProjectWorkspace::Cargo { sysroot, .. }
            | ProjectWorkspace::Json { sysroot, .. }
            | ProjectWorkspace::DetachedFiles { sysroot, .. }) = ws;
            match sysroot {
                Err(None) => (),
                Err(Some(e)) => {
                    status.health = lsp_ext::Health::Warning;
                    message.push_str(e);
                    message.push_str("\n\n");
                }
                Ok(s) => {
                    if let Some(e) = s.loading_warning() {
                        status.health = lsp_ext::Health::Warning;
                        message.push_str(&e);
                        message.push_str("\n\n");
                    }
                }
            }
            if let ProjectWorkspace::Cargo { rustc: Err(Some(e)), .. } = ws {
                status.health = lsp_ext::Health::Warning;
                message.push_str(e);
                message.push_str("\n\n");
            }
        }

        if let Err(_) = self.fetch_workspace_error() {
            status.health = lsp_ext::Health::Error;
            message.push_str("Failed to load workspaces.\n\n");
        }

        if !message.is_empty() {
            status.message = Some(message.trim_end().to_owned());
        }
        status
    }

    pub(crate) fn fetch_workspaces(&mut self, cause: Cause, force_crate_graph_reload: bool) {
        tracing::info!(%cause, "will fetch workspaces");

        self.task_pool.handle.spawn_with_sender(ThreadIntent::Worker, {
            let linked_projects = self.config.linked_projects();
            let detached_files = self.config.detached_files().to_vec();
            let cargo_config = self.config.cargo();

            move |sender| {
                let progress = {
                    let sender = sender.clone();
                    move |msg| {
                        sender
                            .send(Task::FetchWorkspace(ProjectWorkspaceProgress::Report(msg)))
                            .unwrap()
                    }
                };

                sender.send(Task::FetchWorkspace(ProjectWorkspaceProgress::Begin)).unwrap();

                let mut workspaces = linked_projects
                    .iter()
                    .map(|project| match project {
                        LinkedProject::ProjectManifest(manifest) => {
                            project_model::ProjectWorkspace::load(
                                manifest.clone(),
                                &cargo_config,
                                &progress,
                            )
                        }
                        LinkedProject::InlineJsonProject(it) => {
                            Ok(project_model::ProjectWorkspace::load_inline(
                                it.clone(),
                                cargo_config.target.as_deref(),
                                &cargo_config.extra_env,
                                None,
                            ))
                        }
                    })
                    .collect::<Vec<_>>();

                let mut i = 0;
                while i < workspaces.len() {
                    if let Ok(w) = &workspaces[i] {
                        let dupes: Vec<_> = workspaces
                            .iter()
                            .enumerate()
                            .skip(i + 1)
                            .filter_map(|(i, it)| {
                                it.as_ref().ok().filter(|ws| ws.eq_ignore_build_data(w)).map(|_| i)
                            })
                            .collect();
                        dupes.into_iter().rev().for_each(|d| {
                            _ = workspaces.remove(d);
                        });
                    }
                    i += 1;
                }

                if !detached_files.is_empty() {
                    workspaces.push(project_model::ProjectWorkspace::load_detached_files(
                        detached_files,
                        &cargo_config,
                    ));
                }

                tracing::info!("did fetch workspaces {:?}", workspaces);
                sender
                    .send(Task::FetchWorkspace(ProjectWorkspaceProgress::End(
                        workspaces,
                        force_crate_graph_reload,
                    )))
                    .unwrap();
            }
        });
    }

    pub(crate) fn fetch_build_data(&mut self, cause: Cause) {
        tracing::info!(%cause, "will fetch build data");
        let workspaces = Arc::clone(&self.workspaces);
        let config = self.config.cargo();
        self.task_pool.handle.spawn_with_sender(ThreadIntent::Worker, move |sender| {
            sender.send(Task::FetchBuildData(BuildDataProgress::Begin)).unwrap();

            let progress = {
                let sender = sender.clone();
                move |msg| {
                    sender.send(Task::FetchBuildData(BuildDataProgress::Report(msg))).unwrap()
                }
            };
            let res = ProjectWorkspace::run_all_build_scripts(&workspaces, &config, &progress);

            sender.send(Task::FetchBuildData(BuildDataProgress::End((workspaces, res)))).unwrap();
        });
    }

    pub(crate) fn fetch_proc_macros(&mut self, cause: Cause, paths: Vec<ProcMacroPaths>) {
        tracing::info!(%cause, "will load proc macros");
        let dummy_replacements = self.config.dummy_replacements().clone();
        let proc_macro_clients = self.proc_macro_clients.clone();

        self.task_pool.handle.spawn_with_sender(ThreadIntent::Worker, move |sender| {
            sender.send(Task::LoadProcMacros(ProcMacroProgress::Begin)).unwrap();

            let dummy_replacements = &dummy_replacements;
            let progress = {
                let sender = sender.clone();
                &move |msg| {
                    sender.send(Task::LoadProcMacros(ProcMacroProgress::Report(msg))).unwrap()
                }
            };

            let mut res = FxHashMap::default();
            let chain = proc_macro_clients
                .iter()
                .map(|res| res.as_ref().map_err(|e| e.to_string()))
                .chain(iter::repeat_with(|| Err("Proc macros servers are not running".into())));
            for (client, paths) in chain.zip(paths) {
                res.extend(paths.into_iter().map(move |(crate_id, res)| {
                    (
                        crate_id,
                        res.map_or_else(
                            |_| Err("proc macro crate is missing dylib".to_owned()),
                            |(crate_name, path)| {
                                progress(path.to_string());
                                client.as_ref().map_err(Clone::clone).and_then(|client| {
                                    load_proc_macro(
                                        client,
                                        &path,
                                        crate_name
                                            .as_deref()
                                            .and_then(|crate_name| {
                                                dummy_replacements.get(crate_name).map(|v| &**v)
                                            })
                                            .unwrap_or_default(),
                                    )
                                })
                            },
                        ),
                    )
                }));
            }

            sender.send(Task::LoadProcMacros(ProcMacroProgress::End(res))).unwrap();
        });
    }

    pub(crate) fn set_proc_macros(&mut self, proc_macros: ProcMacros) {
        let mut change = Change::new();
        change.set_proc_macros(proc_macros);
        self.analysis_host.apply_change(change);
    }

    pub(crate) fn switch_workspaces(&mut self, cause: Cause) {
        let _p = profile::span("GlobalState::switch_workspaces");
        tracing::info!(%cause, "will switch workspaces");

        let Some((workspaces, force_reload_crate_graph)) =
            self.fetch_workspaces_queue.last_op_result()
        else {
            return;
        };

        if let Err(_) = self.fetch_workspace_error() {
            if !self.workspaces.is_empty() {
                if *force_reload_crate_graph {
                    self.recreate_crate_graph(cause);
                }
                // It only makes sense to switch to a partially broken workspace
                // if we don't have any workspace at all yet.
                return;
            }
        }

        let workspaces =
            workspaces.iter().filter_map(|res| res.as_ref().ok().cloned()).collect::<Vec<_>>();

        let same_workspaces = workspaces.len() == self.workspaces.len()
            && workspaces
                .iter()
                .zip(self.workspaces.iter())
                .all(|(l, r)| l.eq_ignore_build_data(r));

        if same_workspaces {
            let (workspaces, build_scripts) = self.fetch_build_data_queue.last_op_result();
            if Arc::ptr_eq(workspaces, &self.workspaces) {
                tracing::debug!("set build scripts to workspaces");

                let workspaces = workspaces
                    .iter()
                    .cloned()
                    .zip(build_scripts)
                    .map(|(mut ws, bs)| {
                        ws.set_build_scripts(bs.as_ref().ok().cloned().unwrap_or_default());
                        ws
                    })
                    .collect::<Vec<_>>();

                // Workspaces are the same, but we've updated build data.
                self.workspaces = Arc::new(workspaces);
            } else {
                tracing::info!("build scripts do not match the version of the active workspace");
                if *force_reload_crate_graph {
                    self.recreate_crate_graph(cause);
                }
                // Current build scripts do not match the version of the active
                // workspace, so there's nothing for us to update.
                return;
            }
        } else {
            tracing::debug!("abandon build scripts for workspaces");

            // Here, we completely changed the workspace (Cargo.toml edit), so
            // we don't care about build-script results, they are stale.
            // FIXME: can we abort the build scripts here?
            self.workspaces = Arc::new(workspaces);
        }

        if let FilesWatcher::Client = self.config.files().watcher {
            let registration_options = lsp_types::DidChangeWatchedFilesRegistrationOptions {
                watchers: self
                    .workspaces
                    .iter()
                    .flat_map(|ws| ws.to_roots())
                    .filter(|it| it.is_local)
                    .flat_map(|root| {
                        root.include.into_iter().flat_map(|it| {
                            [
                                format!("{it}/**/*.rs"),
                                format!("{it}/**/Cargo.toml"),
                                format!("{it}/**/Cargo.lock"),
                            ]
                        })
                    })
                    .map(|glob_pattern| lsp_types::FileSystemWatcher {
                        glob_pattern: lsp_types::GlobPattern::String(glob_pattern),
                        kind: None,
                    })
                    .collect(),
            };
            let registration = lsp_types::Registration {
                id: "workspace/didChangeWatchedFiles".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(serde_json::to_value(registration_options).unwrap()),
            };
            self.send_request::<lsp_types::request::RegisterCapability>(
                lsp_types::RegistrationParams { registrations: vec![registration] },
                |_, _| (),
            );
        }

        let files_config = self.config.files();
        let project_folders = ProjectFolders::new(&self.workspaces, &files_config.exclude);

        if self.proc_macro_clients.is_empty() || !same_workspaces {
            if self.config.expand_proc_macros() {
                tracing::info!("Spawning proc-macro servers");

                // FIXME: use `Arc::from_iter` when it becomes available
                self.proc_macro_clients = Arc::from(
                    self.workspaces
                        .iter()
                        .map(|ws| {
                            let path = match self.config.proc_macro_srv() {
                                Some(path) => path,
                                None => ws.find_sysroot_proc_macro_srv()?,
                            };

                            tracing::info!("Using proc-macro server at {path}");
                            ProcMacroServer::spawn(path.clone()).map_err(|err| {
                                tracing::error!(
                                    "Failed to run proc-macro server from path {path}, error: {err:?}",
                                );
                                anyhow::format_err!(
                                    "Failed to run proc-macro server from path {path}, error: {err:?}",
                                )
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            };
        }

        let watch = match files_config.watcher {
            FilesWatcher::Client => vec![],
            FilesWatcher::Server => project_folders.watch,
        };
        self.vfs_config_version += 1;
        self.loader.handle.set_config(vfs::loader::Config {
            load: project_folders.load,
            watch,
            version: self.vfs_config_version,
        });
        self.source_root_config = project_folders.source_root_config;

        self.recreate_crate_graph(cause);

        tracing::info!("did switch workspaces");
    }

    fn recreate_crate_graph(&mut self, cause: String) {
        // Create crate graph from all the workspaces
        let (crate_graph, proc_macro_paths, crate_graph_file_dependencies) = {
            let vfs = &mut self.vfs.write().0;
            let loader = &mut self.loader;
            // crate graph construction relies on these paths, record them so when one of them gets
            // deleted or created we trigger a reconstruction of the crate graph
            let mut crate_graph_file_dependencies = FxHashSet::default();

            let mut load = |path: &AbsPath| {
                let _p = profile::span("switch_workspaces::load");
                let vfs_path = vfs::VfsPath::from(path.to_path_buf());
                crate_graph_file_dependencies.insert(vfs_path.clone());
                match vfs.file_id(&vfs_path) {
                    Some(file_id) => Some(file_id),
                    None => {
                        if !self.mem_docs.contains(&vfs_path) {
                            let contents = loader.handle.load_sync(path);
                            vfs.set_file_contents(vfs_path.clone(), contents);
                        }
                        vfs.file_id(&vfs_path)
                    }
                }
            };

            let mut crate_graph = CrateGraph::default();
            let mut proc_macros = Vec::default();
            for ws in &**self.workspaces {
                let (other, mut crate_proc_macros) =
                    ws.to_crate_graph(&mut load, &self.config.extra_env());
                crate_graph.extend(other, &mut crate_proc_macros);
                proc_macros.push(crate_proc_macros);
            }
            (crate_graph, proc_macros, crate_graph_file_dependencies)
        };

        if self.config.expand_proc_macros() {
            self.fetch_proc_macros_queue.request_op(cause, proc_macro_paths);
        }
        let mut change = Change::new();
        change.set_crate_graph(crate_graph);
        self.analysis_host.apply_change(change);
        self.crate_graph_file_dependencies = crate_graph_file_dependencies;
        self.process_changes();

        self.reload_flycheck();
    }

    pub(super) fn fetch_workspace_error(&self) -> Result<(), String> {
        let mut buf = String::new();

        let Some((last_op_result, _)) = self.fetch_workspaces_queue.last_op_result() else {
            return Ok(());
        };
        if last_op_result.is_empty() {
            stdx::format_to!(buf, "rust-analyzer failed to discover workspace");
        } else {
            for ws in last_op_result {
                if let Err(err) = ws {
                    stdx::format_to!(buf, "rust-analyzer failed to load workspace: {:#}\n", err);
                }
            }
        }

        if buf.is_empty() {
            return Ok(());
        }

        Err(buf)
    }

    pub(super) fn fetch_build_data_error(&self) -> Result<(), String> {
        let mut buf = String::new();

        for ws in &self.fetch_build_data_queue.last_op_result().1 {
            match ws {
                Ok(data) => match data.error() {
                    Some(stderr) => stdx::format_to!(buf, "{:#}\n", stderr),
                    _ => (),
                },
                // io errors
                Err(err) => stdx::format_to!(buf, "{:#}\n", err),
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(buf)
        }
    }

    fn reload_flycheck(&mut self) {
        let _p = profile::span("GlobalState::reload_flycheck");
        let config = self.config.flycheck();
        let sender = self.flycheck_sender.clone();
        let invocation_strategy = match config {
            FlycheckConfig::CargoCommand { .. } => flycheck::InvocationStrategy::PerWorkspace,
            FlycheckConfig::CustomCommand { invocation_strategy, .. } => invocation_strategy,
        };

        self.flycheck = match invocation_strategy {
            flycheck::InvocationStrategy::Once => vec![FlycheckHandle::spawn(
                0,
                Box::new(move |msg| sender.send(msg).unwrap()),
                config,
                self.config.root_path().clone(),
            )],
            flycheck::InvocationStrategy::PerWorkspace => {
                self.workspaces
                    .iter()
                    .enumerate()
                    .filter_map(|(id, w)| match w {
                        ProjectWorkspace::Cargo { cargo, .. } => Some((id, cargo.workspace_root())),
                        ProjectWorkspace::Json { project, .. } => {
                            // Enable flychecks for json projects if a custom flycheck command was supplied
                            // in the workspace configuration.
                            match config {
                                FlycheckConfig::CustomCommand { .. } => Some((id, project.path())),
                                _ => None,
                            }
                        }
                        ProjectWorkspace::DetachedFiles { .. } => None,
                    })
                    .map(|(id, root)| {
                        let sender = sender.clone();
                        FlycheckHandle::spawn(
                            id,
                            Box::new(move |msg| sender.send(msg).unwrap()),
                            config.clone(),
                            root.to_path_buf(),
                        )
                    })
                    .collect()
            }
        }
        .into();
    }
}

pub(crate) fn should_refresh_for_change(path: &AbsPath, change_kind: ChangeKind) -> bool {
    const IMPLICIT_TARGET_FILES: &[&str] = &["build.rs", "src/main.rs", "src/lib.rs"];
    const IMPLICIT_TARGET_DIRS: &[&str] = &["src/bin", "examples", "tests", "benches"];

    let file_name = match path.file_name().unwrap_or_default().to_str() {
        Some(it) => it,
        None => return false,
    };

    if let "Cargo.toml" | "Cargo.lock" = file_name {
        return true;
    }
    if change_kind == ChangeKind::Modify {
        return false;
    }

    // .cargo/config{.toml}
    if path.extension().unwrap_or_default() != "rs" {
        let is_cargo_config = matches!(file_name, "config.toml" | "config")
            && path.parent().map(|parent| parent.as_ref().ends_with(".cargo")).unwrap_or(false);
        return is_cargo_config;
    }

    if IMPLICIT_TARGET_FILES.iter().any(|it| path.as_ref().ends_with(it)) {
        return true;
    }
    let parent = match path.parent() {
        Some(it) => it,
        None => return false,
    };
    if IMPLICIT_TARGET_DIRS.iter().any(|it| parent.as_ref().ends_with(it)) {
        return true;
    }
    if file_name == "main.rs" {
        let grand_parent = match parent.parent() {
            Some(it) => it,
            None => return false,
        };
        if IMPLICIT_TARGET_DIRS.iter().any(|it| grand_parent.as_ref().ends_with(it)) {
            return true;
        }
    }
    false
}
