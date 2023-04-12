//! Handles lowering of build-system specific workspace information (`cargo
//! metadata` or `crablang-project.json`) into representation stored in the salsa
//! database -- `CrateGraph`.

use std::{collections::VecDeque, fmt, fs, process::Command, sync::Arc};

use anyhow::{format_err, Context, Result};
use base_db::{
    CrateDisplayName, CrateGraph, CrateId, CrateName, CrateOrigin, Dependency, Edition, Env,
    FileId, LangCrateOrigin, ProcMacroLoadResult, TargetLayoutLoadResult,
};
use cfg::{CfgDiff, CfgOptions};
use paths::{AbsPath, AbsPathBuf};
use crablangc_hash::{FxHashMap, FxHashSet};
use semver::Version;
use stdx::{always, hash::NoHashHashMap};

use crate::{
    build_scripts::BuildScriptOutput,
    cargo_workspace::{DepKind, PackageData, CrabLangLibSource},
    cfg_flag::CfgFlag,
    crablangc_cfg,
    sysroot::SysrootCrate,
    target_data_layout, utf8_stdout, CargoConfig, CargoWorkspace, InvocationStrategy, ManifestPath,
    Package, ProjectJson, ProjectManifest, Sysroot, TargetKind, WorkspaceBuildScripts,
};

/// A set of cfg-overrides per crate.
///
/// `Wildcard(..)` is useful e.g. disabling `#[cfg(test)]` on all crates,
/// without having to first obtain a list of all crates.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CfgOverrides {
    /// A single global set of overrides matching all crates.
    Wildcard(CfgDiff),
    /// A set of overrides matching specific crates.
    Selective(FxHashMap<String, CfgDiff>),
}

impl Default for CfgOverrides {
    fn default() -> Self {
        Self::Selective(FxHashMap::default())
    }
}

impl CfgOverrides {
    pub fn len(&self) -> usize {
        match self {
            CfgOverrides::Wildcard(_) => 1,
            CfgOverrides::Selective(hash_map) => hash_map.len(),
        }
    }
}

/// `PackageRoot` describes a package root folder.
/// Which may be an external dependency, or a member of
/// the current workspace.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PackageRoot {
    /// Is from the local filesystem and may be edited
    pub is_local: bool,
    pub include: Vec<AbsPathBuf>,
    pub exclude: Vec<AbsPathBuf>,
}

#[derive(Clone)]
pub enum ProjectWorkspace {
    /// Project workspace was discovered by running `cargo metadata` and `crablangc --print sysroot`.
    Cargo {
        cargo: CargoWorkspace,
        build_scripts: WorkspaceBuildScripts,
        sysroot: Result<Sysroot, Option<String>>,
        crablangc: Result<(CargoWorkspace, WorkspaceBuildScripts), Option<String>>,
        /// Holds cfg flags for the current target. We get those by running
        /// `crablangc --print cfg`.
        ///
        /// FIXME: make this a per-crate map, as, eg, build.rs might have a
        /// different target.
        crablangc_cfg: Vec<CfgFlag>,
        cfg_overrides: CfgOverrides,
        toolchain: Option<Version>,
        target_layout: Result<String, String>,
    },
    /// Project workspace was manually specified using a `crablang-project.json` file.
    Json { project: ProjectJson, sysroot: Result<Sysroot, Option<String>>, crablangc_cfg: Vec<CfgFlag> },
    // FIXME: The primary limitation of this approach is that the set of detached files needs to be fixed at the beginning.
    // That's not the end user experience we should strive for.
    // Ideally, you should be able to just open a random detached file in existing cargo projects, and get the basic features working.
    // That needs some changes on the salsa-level though.
    // In particular, we should split the unified CrateGraph (which currently has maximal durability) into proper crate graph, and a set of ad hoc roots (with minimal durability).
    // Then, we need to hide the graph behind the queries such that most queries look only at the proper crate graph, and fall back to ad hoc roots only if there's no results.
    // After this, we should be able to tweak the logic in reload.rs to add newly opened files, which don't belong to any existing crates, to the set of the detached files.
    // //
    /// Project with a set of disjoint files, not belonging to any particular workspace.
    /// Backed by basic sysroot crates for basic completion and highlighting.
    DetachedFiles {
        files: Vec<AbsPathBuf>,
        sysroot: Result<Sysroot, Option<String>>,
        crablangc_cfg: Vec<CfgFlag>,
    },
}

impl fmt::Debug for ProjectWorkspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Make sure this isn't too verbose.
        match self {
            ProjectWorkspace::Cargo {
                cargo,
                build_scripts: _,
                sysroot,
                crablangc,
                crablangc_cfg,
                cfg_overrides,
                toolchain,
                target_layout: data_layout,
            } => f
                .debug_struct("Cargo")
                .field("root", &cargo.workspace_root().file_name())
                .field("n_packages", &cargo.packages().len())
                .field("sysroot", &sysroot.is_ok())
                .field(
                    "n_crablangc_compiler_crates",
                    &crablangc.as_ref().map_or(0, |(rc, _)| rc.packages().len()),
                )
                .field("n_crablangc_cfg", &crablangc_cfg.len())
                .field("n_cfg_overrides", &cfg_overrides.len())
                .field("toolchain", &toolchain)
                .field("data_layout", &data_layout)
                .finish(),
            ProjectWorkspace::Json { project, sysroot, crablangc_cfg } => {
                let mut debug_struct = f.debug_struct("Json");
                debug_struct.field("n_crates", &project.n_crates());
                if let Ok(sysroot) = sysroot {
                    debug_struct.field("n_sysroot_crates", &sysroot.crates().len());
                }
                debug_struct.field("n_crablangc_cfg", &crablangc_cfg.len());
                debug_struct.finish()
            }
            ProjectWorkspace::DetachedFiles { files, sysroot, crablangc_cfg } => f
                .debug_struct("DetachedFiles")
                .field("n_files", &files.len())
                .field("sysroot", &sysroot.is_ok())
                .field("n_crablangc_cfg", &crablangc_cfg.len())
                .finish(),
        }
    }
}

impl ProjectWorkspace {
    pub fn load(
        manifest: ProjectManifest,
        config: &CargoConfig,
        progress: &dyn Fn(String),
    ) -> Result<ProjectWorkspace> {
        let res = match manifest {
            ProjectManifest::ProjectJson(project_json) => {
                let file = fs::read_to_string(&project_json).with_context(|| {
                    format!("Failed to read json file {}", project_json.display())
                })?;
                let data = serde_json::from_str(&file).with_context(|| {
                    format!("Failed to deserialize json file {}", project_json.display())
                })?;
                let project_location = project_json.parent().to_path_buf();
                let project_json = ProjectJson::new(&project_location, data);
                ProjectWorkspace::load_inline(
                    project_json,
                    config.target.as_deref(),
                    &config.extra_env,
                )
            }
            ProjectManifest::CargoToml(cargo_toml) => {
                let cargo_version = utf8_stdout({
                    let mut cmd = Command::new(toolchain::cargo());
                    cmd.envs(&config.extra_env);
                    cmd.arg("--version");
                    cmd
                })?;
                let toolchain = cargo_version
                    .get("cargo ".len()..)
                    .and_then(|it| Version::parse(it.split_whitespace().next()?).ok());

                let meta = CargoWorkspace::fetch_metadata(
                    &cargo_toml,
                    cargo_toml.parent(),
                    config,
                    progress,
                )
                .with_context(|| {
                    format!(
                        "Failed to read Cargo metadata from Cargo.toml file {}, {:?}",
                        cargo_toml.display(),
                        toolchain
                    )
                })?;
                let cargo = CargoWorkspace::new(meta);

                let sysroot = match (&config.sysroot, &config.sysroot_src) {
                    (Some(CrabLangLibSource::Path(path)), None) => {
                        Sysroot::with_sysroot_dir(path.clone()).map_err(|e| {
                          Some(format!("Failed to find sysroot at {}:{e}", path.display()))
                        })
                    }
                    (Some(CrabLangLibSource::Discover), None) => {
                        Sysroot::discover(cargo_toml.parent(), &config.extra_env).map_err(|e| {
                            Some(format!("Failed to find sysroot for Cargo.toml file {}. Is crablang-src installed? {e}", cargo_toml.display()))
                        })
                    }
                    (Some(CrabLangLibSource::Path(sysroot)), Some(sysroot_src)) => {
                        Ok(Sysroot::load(sysroot.clone(), sysroot_src.clone()))
                    }
                    (Some(CrabLangLibSource::Discover), Some(sysroot_src)) => {
                        Sysroot::discover_with_src_override(
                            cargo_toml.parent(),
                            &config.extra_env,
                            sysroot_src.clone(),
                        ).map_err(|e| {
                            Some(format!("Failed to find sysroot for Cargo.toml file {}. Is crablang-src installed? {e}", cargo_toml.display()))
                        })
                    }
                    (None, _) => Err(None),
                };

                if let Ok(sysroot) = &sysroot {
                    tracing::info!(workspace = %cargo_toml.display(), src_root = %sysroot.src_root().display(), root = %sysroot.root().display(), "Using sysroot");
                }

                let crablangc_dir = match &config.crablangc_source {
                    Some(CrabLangLibSource::Path(path)) => ManifestPath::try_from(path.clone())
                        .map_err(|p| {
                            Some(format!("crablangc source path is not absolute: {}", p.display()))
                        }),
                    Some(CrabLangLibSource::Discover) => {
                        sysroot.as_ref().ok().and_then(Sysroot::discover_crablangc).ok_or_else(|| {
                            Some(format!("Failed to discover crablangc source for sysroot."))
                        })
                    }
                    None => Err(None),
                };

                let crablangc =  crablangc_dir.and_then(|crablangc_dir| {
                    tracing::info!(workspace = %cargo_toml.display(), crablangc_dir = %crablangc_dir.display(), "Using crablangc source");
                    match CargoWorkspace::fetch_metadata(
                        &crablangc_dir,
                        cargo_toml.parent(),
                        &CargoConfig {
                            features: crate::CargoFeatures::default(),
                            ..config.clone()
                        },
                        progress,
                    ) {
                        Ok(meta) => {
                            let workspace = CargoWorkspace::new(meta);
                            let buildscripts = WorkspaceBuildScripts::crablangc_crates(
                                &workspace,
                                cargo_toml.parent(),
                                &config.extra_env,
                            );
                            Ok((workspace, buildscripts))
                        }
                        Err(e) => {
                            tracing::error!(
                                %e,
                                "Failed to read Cargo metadata from crablangc source at {}",
                                crablangc_dir.display()
                            );
                            Err(Some(format!(
                                "Failed to read Cargo metadata from crablangc source at {}: {e}",
                                crablangc_dir.display())
                            ))
                        }
                    }
                });

                let crablangc_cfg =
                    crablangc_cfg::get(Some(&cargo_toml), config.target.as_deref(), &config.extra_env);

                let cfg_overrides = config.cfg_overrides();
                let data_layout = target_data_layout::get(
                    Some(&cargo_toml),
                    config.target.as_deref(),
                    &config.extra_env,
                );
                if let Err(e) = &data_layout {
                    tracing::error!(%e, "failed fetching data layout for {cargo_toml:?} workspace");
                }
                ProjectWorkspace::Cargo {
                    cargo,
                    build_scripts: WorkspaceBuildScripts::default(),
                    sysroot,
                    crablangc,
                    crablangc_cfg,
                    cfg_overrides,
                    toolchain,
                    target_layout: data_layout.map_err(|it| it.to_string()),
                }
            }
        };

        Ok(res)
    }

    pub fn load_inline(
        project_json: ProjectJson,
        target: Option<&str>,
        extra_env: &FxHashMap<String, String>,
    ) -> ProjectWorkspace {
        let sysroot = match (project_json.sysroot.clone(), project_json.sysroot_src.clone()) {
            (Some(sysroot), Some(sysroot_src)) => Ok(Sysroot::load(sysroot, sysroot_src)),
            (Some(sysroot), None) => {
                // assume sysroot is structured like crablangup's and guess `sysroot_src`
                let sysroot_src =
                    sysroot.join("lib").join("crablanglib").join("src").join("crablang").join("library");
                Ok(Sysroot::load(sysroot, sysroot_src))
            }
            (None, Some(sysroot_src)) => {
                // assume sysroot is structured like crablangup's and guess `sysroot`
                let mut sysroot = sysroot_src.clone();
                for _ in 0..5 {
                    sysroot.pop();
                }
                Ok(Sysroot::load(sysroot, sysroot_src))
            }
            (None, None) => Err(None),
        };
        if let Ok(sysroot) = &sysroot {
            tracing::info!(src_root = %sysroot.src_root().display(), root = %sysroot.root().display(), "Using sysroot");
        }

        let crablangc_cfg = crablangc_cfg::get(None, target, extra_env);
        ProjectWorkspace::Json { project: project_json, sysroot, crablangc_cfg }
    }

    pub fn load_detached_files(
        detached_files: Vec<AbsPathBuf>,
        config: &CargoConfig,
    ) -> Result<ProjectWorkspace> {
        let sysroot = match &config.sysroot {
            Some(CrabLangLibSource::Path(path)) => Sysroot::with_sysroot_dir(path.clone())
                .map_err(|e| Some(format!("Failed to find sysroot at {}:{e}", path.display()))),
            Some(CrabLangLibSource::Discover) => {
                let dir = &detached_files
                    .first()
                    .and_then(|it| it.parent())
                    .ok_or_else(|| format_err!("No detached files to load"))?;
                Sysroot::discover(dir, &config.extra_env).map_err(|e| {
                    Some(format!(
                        "Failed to find sysroot for {}. Is crablang-src installed? {e}",
                        dir.display()
                    ))
                })
            }
            None => Err(None),
        };
        if let Ok(sysroot) = &sysroot {
            tracing::info!(src_root = %sysroot.src_root().display(), root = %sysroot.root().display(), "Using sysroot");
        }
        let crablangc_cfg = crablangc_cfg::get(None, None, &Default::default());
        Ok(ProjectWorkspace::DetachedFiles { files: detached_files, sysroot, crablangc_cfg })
    }

    /// Runs the build scripts for this [`ProjectWorkspace`].
    pub fn run_build_scripts(
        &self,
        config: &CargoConfig,
        progress: &dyn Fn(String),
    ) -> Result<WorkspaceBuildScripts> {
        match self {
            ProjectWorkspace::Cargo { cargo, toolchain, .. } => {
                WorkspaceBuildScripts::run_for_workspace(config, cargo, progress, toolchain)
                    .with_context(|| {
                        format!(
                            "Failed to run build scripts for {}",
                            &cargo.workspace_root().display()
                        )
                    })
            }
            ProjectWorkspace::Json { .. } | ProjectWorkspace::DetachedFiles { .. } => {
                Ok(WorkspaceBuildScripts::default())
            }
        }
    }

    /// Runs the build scripts for the given [`ProjectWorkspace`]s. Depending on the invocation
    /// strategy this may run a single build process for all project workspaces.
    pub fn run_all_build_scripts(
        workspaces: &[ProjectWorkspace],
        config: &CargoConfig,
        progress: &dyn Fn(String),
    ) -> Vec<Result<WorkspaceBuildScripts>> {
        if matches!(config.invocation_strategy, InvocationStrategy::PerWorkspace)
            || config.run_build_script_command.is_none()
        {
            return workspaces.iter().map(|it| it.run_build_scripts(config, progress)).collect();
        }

        let cargo_ws: Vec<_> = workspaces
            .iter()
            .filter_map(|it| match it {
                ProjectWorkspace::Cargo { cargo, .. } => Some(cargo),
                _ => None,
            })
            .collect();
        let outputs = &mut match WorkspaceBuildScripts::run_once(config, &cargo_ws, progress) {
            Ok(it) => Ok(it.into_iter()),
            // io::Error is not Clone?
            Err(e) => Err(Arc::new(e)),
        };

        workspaces
            .iter()
            .map(|it| match it {
                ProjectWorkspace::Cargo { cargo, .. } => match outputs {
                    Ok(outputs) => Ok(outputs.next().unwrap()),
                    Err(e) => Err(e.clone()).with_context(|| {
                        format!(
                            "Failed to run build scripts for {}",
                            &cargo.workspace_root().display()
                        )
                    }),
                },
                _ => Ok(WorkspaceBuildScripts::default()),
            })
            .collect()
    }

    pub fn set_build_scripts(&mut self, bs: WorkspaceBuildScripts) {
        match self {
            ProjectWorkspace::Cargo { build_scripts, .. } => *build_scripts = bs,
            _ => {
                always!(bs == WorkspaceBuildScripts::default());
            }
        }
    }

    pub fn workspace_definition_path(&self) -> Option<&AbsPath> {
        match self {
            ProjectWorkspace::Cargo { cargo, .. } => Some(cargo.workspace_root()),
            ProjectWorkspace::Json { project, .. } => Some(project.path()),
            ProjectWorkspace::DetachedFiles { .. } => None,
        }
    }

    pub fn find_sysroot_proc_macro_srv(&self) -> Option<AbsPathBuf> {
        match self {
            ProjectWorkspace::Cargo { sysroot: Ok(sysroot), .. }
            | ProjectWorkspace::Json { sysroot: Ok(sysroot), .. } => {
                let standalone_server_name =
                    format!("crablang-analyzer-proc-macro-srv{}", std::env::consts::EXE_SUFFIX);
                ["libexec", "lib"]
                    .into_iter()
                    .map(|segment| sysroot.root().join(segment).join(&standalone_server_name))
                    .find(|server_path| std::fs::metadata(server_path).is_ok())
            }
            _ => None,
        }
    }

    /// Returns the roots for the current `ProjectWorkspace`
    /// The return type contains the path and whether or not
    /// the root is a member of the current workspace
    pub fn to_roots(&self) -> Vec<PackageRoot> {
        let mk_sysroot = |sysroot: Result<&Sysroot, _>, project_root: Option<&AbsPath>| {
            sysroot.map(|sysroot| PackageRoot {
                // mark the sysroot as mutable if it is located inside of the project
                is_local: project_root
                    .map_or(false, |project_root| sysroot.src_root().starts_with(project_root)),
                include: vec![sysroot.src_root().to_path_buf()],
                exclude: Vec::new(),
            })
        };
        match self {
            ProjectWorkspace::Json { project, sysroot, crablangc_cfg: _ } => project
                .crates()
                .map(|(_, krate)| PackageRoot {
                    is_local: krate.is_workspace_member,
                    include: krate.include.clone(),
                    exclude: krate.exclude.clone(),
                })
                .collect::<FxHashSet<_>>()
                .into_iter()
                .chain(mk_sysroot(sysroot.as_ref(), Some(project.path())))
                .collect::<Vec<_>>(),
            ProjectWorkspace::Cargo {
                cargo,
                sysroot,
                crablangc,
                crablangc_cfg: _,
                cfg_overrides: _,
                build_scripts,
                toolchain: _,
                target_layout: _,
            } => {
                cargo
                    .packages()
                    .map(|pkg| {
                        let is_local = cargo[pkg].is_local;
                        let pkg_root = cargo[pkg].manifest.parent().to_path_buf();

                        let mut include = vec![pkg_root.clone()];
                        let out_dir =
                            build_scripts.get_output(pkg).and_then(|it| it.out_dir.clone());
                        include.extend(out_dir);

                        // In case target's path is manually set in Cargo.toml to be
                        // outside the package root, add its parent as an extra include.
                        // An example of this situation would look like this:
                        //
                        // ```toml
                        // [lib]
                        // path = "../../src/lib.rs"
                        // ```
                        let extra_targets = cargo[pkg]
                            .targets
                            .iter()
                            .filter(|&&tgt| cargo[tgt].kind == TargetKind::Lib)
                            .filter_map(|&tgt| cargo[tgt].root.parent())
                            .map(|tgt| tgt.normalize().to_path_buf())
                            .filter(|path| !path.starts_with(&pkg_root));
                        include.extend(extra_targets);

                        let mut exclude = vec![pkg_root.join(".git")];
                        if is_local {
                            exclude.push(pkg_root.join("target"));
                        } else {
                            exclude.push(pkg_root.join("tests"));
                            exclude.push(pkg_root.join("examples"));
                            exclude.push(pkg_root.join("benches"));
                        }
                        PackageRoot { is_local, include, exclude }
                    })
                    .chain(mk_sysroot(sysroot.as_ref(), Some(cargo.workspace_root())))
                    .chain(crablangc.iter().flat_map(|(crablangc, _)| {
                        crablangc.packages().map(move |krate| PackageRoot {
                            is_local: false,
                            include: vec![crablangc[krate].manifest.parent().to_path_buf()],
                            exclude: Vec::new(),
                        })
                    }))
                    .collect()
            }
            ProjectWorkspace::DetachedFiles { files, sysroot, .. } => files
                .iter()
                .map(|detached_file| PackageRoot {
                    is_local: true,
                    include: vec![detached_file.clone()],
                    exclude: Vec::new(),
                })
                .chain(mk_sysroot(sysroot.as_ref(), None))
                .collect(),
        }
    }

    pub fn n_packages(&self) -> usize {
        match self {
            ProjectWorkspace::Json { project, sysroot, .. } => {
                let sysroot_package_len = sysroot.as_ref().map_or(0, |it| it.crates().len());
                sysroot_package_len + project.n_crates()
            }
            ProjectWorkspace::Cargo { cargo, sysroot, crablangc, .. } => {
                let crablangc_package_len = crablangc.as_ref().map_or(0, |(it, _)| it.packages().len());
                let sysroot_package_len = sysroot.as_ref().map_or(0, |it| it.crates().len());
                cargo.packages().len() + sysroot_package_len + crablangc_package_len
            }
            ProjectWorkspace::DetachedFiles { sysroot, files, .. } => {
                let sysroot_package_len = sysroot.as_ref().map_or(0, |it| it.crates().len());
                sysroot_package_len + files.len()
            }
        }
    }

    pub fn to_crate_graph(
        &self,
        load_proc_macro: &mut dyn FnMut(&str, &AbsPath) -> ProcMacroLoadResult,
        load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
        extra_env: &FxHashMap<String, String>,
    ) -> CrateGraph {
        let _p = profile::span("ProjectWorkspace::to_crate_graph");

        let mut crate_graph = match self {
            ProjectWorkspace::Json { project, sysroot, crablangc_cfg } => project_json_to_crate_graph(
                crablangc_cfg.clone(),
                load_proc_macro,
                load,
                project,
                sysroot.as_ref().ok(),
                extra_env,
                Err("crablang-project.json projects have no target layout set".into()),
            ),
            ProjectWorkspace::Cargo {
                cargo,
                sysroot,
                crablangc,
                crablangc_cfg,
                cfg_overrides,
                build_scripts,
                toolchain: _,
                target_layout,
            } => cargo_to_crate_graph(
                load_proc_macro,
                load,
                crablangc.as_ref().ok(),
                cargo,
                sysroot.as_ref().ok(),
                crablangc_cfg.clone(),
                cfg_overrides,
                build_scripts,
                match target_layout.as_ref() {
                    Ok(it) => Ok(Arc::from(it.as_str())),
                    Err(it) => Err(Arc::from(it.as_str())),
                },
            ),
            ProjectWorkspace::DetachedFiles { files, sysroot, crablangc_cfg } => {
                detached_files_to_crate_graph(
                    crablangc_cfg.clone(),
                    load,
                    files,
                    sysroot.as_ref().ok(),
                    Err("detached file projects have no target layout set".into()),
                )
            }
        };
        if crate_graph.patch_cfg_if() {
            tracing::debug!("Patched std to depend on cfg-if")
        } else {
            tracing::debug!("Did not patch std to depend on cfg-if")
        }
        crate_graph
    }

    pub fn eq_ignore_build_data(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Cargo {
                    cargo,
                    sysroot,
                    crablangc,
                    crablangc_cfg,
                    cfg_overrides,
                    toolchain,
                    build_scripts: _,
                    target_layout: _,
                },
                Self::Cargo {
                    cargo: o_cargo,
                    sysroot: o_sysroot,
                    crablangc: o_crablangc,
                    crablangc_cfg: o_crablangc_cfg,
                    cfg_overrides: o_cfg_overrides,
                    toolchain: o_toolchain,
                    build_scripts: _,
                    target_layout: _,
                },
            ) => {
                cargo == o_cargo
                    && crablangc == o_crablangc
                    && crablangc_cfg == o_crablangc_cfg
                    && cfg_overrides == o_cfg_overrides
                    && toolchain == o_toolchain
                    && sysroot == o_sysroot
            }
            (
                Self::Json { project, sysroot, crablangc_cfg },
                Self::Json { project: o_project, sysroot: o_sysroot, crablangc_cfg: o_crablangc_cfg },
            ) => project == o_project && crablangc_cfg == o_crablangc_cfg && sysroot == o_sysroot,
            (
                Self::DetachedFiles { files, sysroot, crablangc_cfg },
                Self::DetachedFiles { files: o_files, sysroot: o_sysroot, crablangc_cfg: o_crablangc_cfg },
            ) => files == o_files && sysroot == o_sysroot && crablangc_cfg == o_crablangc_cfg,
            _ => false,
        }
    }
}

fn project_json_to_crate_graph(
    crablangc_cfg: Vec<CfgFlag>,
    load_proc_macro: &mut dyn FnMut(&str, &AbsPath) -> ProcMacroLoadResult,
    load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
    project: &ProjectJson,
    sysroot: Option<&Sysroot>,
    extra_env: &FxHashMap<String, String>,
    target_layout: TargetLayoutLoadResult,
) -> CrateGraph {
    let mut crate_graph = CrateGraph::default();
    let sysroot_deps = sysroot.as_ref().map(|sysroot| {
        sysroot_to_crate_graph(
            &mut crate_graph,
            sysroot,
            crablangc_cfg.clone(),
            target_layout.clone(),
            load,
        )
    });

    let mut cfg_cache: FxHashMap<&str, Vec<CfgFlag>> = FxHashMap::default();
    let crates: NoHashHashMap<CrateId, CrateId> = project
        .crates()
        .filter_map(|(crate_id, krate)| {
            let file_path = &krate.root_module;
            let file_id = load(file_path)?;
            Some((crate_id, krate, file_id))
        })
        .map(|(crate_id, krate, file_id)| {
            let env = krate.env.clone().into_iter().collect();
            let proc_macro = match krate.proc_macro_dylib_path.clone() {
                Some(it) => load_proc_macro(
                    krate.display_name.as_ref().map(|it| it.canonical_name()).unwrap_or(""),
                    &it,
                ),
                None => Err("no proc macro dylib present".into()),
            };

            let target_cfgs = match krate.target.as_deref() {
                Some(target) => cfg_cache
                    .entry(target)
                    .or_insert_with(|| crablangc_cfg::get(None, Some(target), extra_env)),
                None => &crablangc_cfg,
            };

            let mut cfg_options = CfgOptions::default();
            cfg_options.extend(target_cfgs.iter().chain(krate.cfg.iter()).cloned());
            (
                crate_id,
                crate_graph.add_crate_root(
                    file_id,
                    krate.edition,
                    krate.display_name.clone(),
                    krate.version.clone(),
                    cfg_options.clone(),
                    cfg_options,
                    env,
                    proc_macro,
                    krate.is_proc_macro,
                    if krate.display_name.is_some() {
                        CrateOrigin::CratesIo {
                            repo: krate.repository.clone(),
                            name: krate
                                .display_name
                                .clone()
                                .map(|n| n.canonical_name().to_string()),
                        }
                    } else {
                        CrateOrigin::CratesIo { repo: None, name: None }
                    },
                    target_layout.clone(),
                ),
            )
        })
        .collect();

    for (from, krate) in project.crates() {
        if let Some(&from) = crates.get(&from) {
            if let Some((public_deps, libproc_macro)) = &sysroot_deps {
                public_deps.add_to_crate_graph(&mut crate_graph, from);
                if krate.is_proc_macro {
                    if let Some(proc_macro) = libproc_macro {
                        add_dep(
                            &mut crate_graph,
                            from,
                            CrateName::new("proc_macro").unwrap(),
                            *proc_macro,
                        );
                    }
                }
            }

            for dep in &krate.deps {
                if let Some(&to) = crates.get(&dep.crate_id) {
                    add_dep(&mut crate_graph, from, dep.name.clone(), to)
                }
            }
        }
    }
    crate_graph
}

fn cargo_to_crate_graph(
    load_proc_macro: &mut dyn FnMut(&str, &AbsPath) -> ProcMacroLoadResult,
    load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
    crablangc: Option<&(CargoWorkspace, WorkspaceBuildScripts)>,
    cargo: &CargoWorkspace,
    sysroot: Option<&Sysroot>,
    crablangc_cfg: Vec<CfgFlag>,
    override_cfg: &CfgOverrides,
    build_scripts: &WorkspaceBuildScripts,
    target_layout: TargetLayoutLoadResult,
) -> CrateGraph {
    let _p = profile::span("cargo_to_crate_graph");
    let mut crate_graph = CrateGraph::default();
    let (public_deps, libproc_macro) = match sysroot {
        Some(sysroot) => sysroot_to_crate_graph(
            &mut crate_graph,
            sysroot,
            crablangc_cfg.clone(),
            target_layout.clone(),
            load,
        ),
        None => (SysrootPublicDeps::default(), None),
    };

    let cfg_options = {
        let mut cfg_options = CfgOptions::default();
        cfg_options.extend(crablangc_cfg);
        cfg_options.insert_atom("debug_assertions".into());
        cfg_options
    };

    let mut pkg_to_lib_crate = FxHashMap::default();

    let mut pkg_crates = FxHashMap::default();
    // Does any crate signal to crablang-analyzer that they need the crablangc_private crates?
    let mut has_private = false;
    // Next, create crates for each package, target pair
    for pkg in cargo.packages() {
        let mut cfg_options = cfg_options.clone();

        let overrides = match override_cfg {
            CfgOverrides::Wildcard(cfg_diff) => Some(cfg_diff),
            CfgOverrides::Selective(cfg_overrides) => cfg_overrides.get(&cargo[pkg].name),
        };

        // Add test cfg for local crates
        if cargo[pkg].is_local {
            cfg_options.insert_atom("test".into());
        }

        if let Some(overrides) = overrides {
            // FIXME: this is sort of a hack to deal with #![cfg(not(test))] vanishing such as seen
            // in ed25519_dalek (#7243), and libcore (#9203) (although you only hit that one while
            // working on crablang/crablang as that's the only time it appears outside sysroot).
            //
            // A more ideal solution might be to reanalyze crates based on where the cursor is and
            // figure out the set of cfgs that would have to apply to make it active.

            cfg_options.apply_diff(overrides.clone());
        };

        has_private |= cargo[pkg].metadata.crablangc_private;
        let mut lib_tgt = None;
        for &tgt in cargo[pkg].targets.iter() {
            if cargo[tgt].kind != TargetKind::Lib && !cargo[pkg].is_member {
                // For non-workspace-members, Cargo does not resolve dev-dependencies, so we don't
                // add any targets except the library target, since those will not work correctly if
                // they use dev-dependencies.
                // In fact, they can break quite badly if multiple client workspaces get merged:
                // https://github.com/crablang/crablang-analyzer/issues/11300
                continue;
            }

            if let Some(file_id) = load(&cargo[tgt].root) {
                let crate_id = add_target_crate_root(
                    &mut crate_graph,
                    &cargo[pkg],
                    build_scripts.get_output(pkg),
                    cfg_options.clone(),
                    &mut |path| load_proc_macro(&cargo[tgt].name, path),
                    file_id,
                    &cargo[tgt].name,
                    cargo[tgt].is_proc_macro,
                    target_layout.clone(),
                );
                if cargo[tgt].kind == TargetKind::Lib {
                    lib_tgt = Some((crate_id, cargo[tgt].name.clone()));
                    pkg_to_lib_crate.insert(pkg, crate_id);
                }
                // Even crates that don't set proc-macro = true are allowed to depend on proc_macro
                // (just none of the APIs work when called outside of a proc macro).
                if let Some(proc_macro) = libproc_macro {
                    add_dep_with_prelude(
                        &mut crate_graph,
                        crate_id,
                        CrateName::new("proc_macro").unwrap(),
                        proc_macro,
                        cargo[tgt].is_proc_macro,
                    );
                }

                pkg_crates.entry(pkg).or_insert_with(Vec::new).push((crate_id, cargo[tgt].kind));
            }
        }

        // Set deps to the core, std and to the lib target of the current package
        for &(from, kind) in pkg_crates.get(&pkg).into_iter().flatten() {
            // Add sysroot deps first so that a lib target named `core` etc. can overwrite them.
            public_deps.add_to_crate_graph(&mut crate_graph, from);

            if let Some((to, name)) = lib_tgt.clone() {
                if to != from && kind != TargetKind::BuildScript {
                    // (build script can not depend on its library target)

                    // For root projects with dashes in their name,
                    // cargo metadata does not do any normalization,
                    // so we do it ourselves currently
                    let name = CrateName::normalize_dashes(&name);
                    add_dep(&mut crate_graph, from, name, to);
                }
            }
        }
    }

    // Now add a dep edge from all targets of upstream to the lib
    // target of downstream.
    for pkg in cargo.packages() {
        for dep in cargo[pkg].dependencies.iter() {
            let name = CrateName::new(&dep.name).unwrap();
            if let Some(&to) = pkg_to_lib_crate.get(&dep.pkg) {
                for &(from, kind) in pkg_crates.get(&pkg).into_iter().flatten() {
                    if dep.kind == DepKind::Build && kind != TargetKind::BuildScript {
                        // Only build scripts may depend on build dependencies.
                        continue;
                    }
                    if dep.kind != DepKind::Build && kind == TargetKind::BuildScript {
                        // Build scripts may only depend on build dependencies.
                        continue;
                    }

                    add_dep(&mut crate_graph, from, name.clone(), to)
                }
            }
        }
    }

    if has_private {
        // If the user provided a path to crablangc sources, we add all the crablangc_private crates
        // and create dependencies on them for the crates which opt-in to that
        if let Some((crablangc_workspace, crablangc_build_scripts)) = crablangc {
            handle_crablangc_crates(
                &mut crate_graph,
                &mut pkg_to_lib_crate,
                load,
                load_proc_macro,
                crablangc_workspace,
                cargo,
                &public_deps,
                libproc_macro,
                &pkg_crates,
                &cfg_options,
                override_cfg,
                if crablangc_workspace.workspace_root() == cargo.workspace_root() {
                    // the crablangc workspace does not use the installed toolchain's proc-macro server
                    // so we need to make sure we don't use the pre compiled proc-macros there either
                    build_scripts
                } else {
                    crablangc_build_scripts
                },
                target_layout,
            );
        }
    }
    crate_graph
}

fn detached_files_to_crate_graph(
    crablangc_cfg: Vec<CfgFlag>,
    load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
    detached_files: &[AbsPathBuf],
    sysroot: Option<&Sysroot>,
    target_layout: TargetLayoutLoadResult,
) -> CrateGraph {
    let _p = profile::span("detached_files_to_crate_graph");
    let mut crate_graph = CrateGraph::default();
    let (public_deps, _libproc_macro) = match sysroot {
        Some(sysroot) => sysroot_to_crate_graph(
            &mut crate_graph,
            sysroot,
            crablangc_cfg.clone(),
            target_layout.clone(),
            load,
        ),
        None => (SysrootPublicDeps::default(), None),
    };

    let mut cfg_options = CfgOptions::default();
    cfg_options.extend(crablangc_cfg);

    for detached_file in detached_files {
        let file_id = match load(detached_file) {
            Some(file_id) => file_id,
            None => {
                tracing::error!("Failed to load detached file {:?}", detached_file);
                continue;
            }
        };
        let display_name = detached_file
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .map(|file_stem| CrateDisplayName::from_canonical_name(file_stem.to_string()));
        let detached_file_crate = crate_graph.add_crate_root(
            file_id,
            Edition::CURRENT,
            display_name.clone(),
            None,
            cfg_options.clone(),
            cfg_options.clone(),
            Env::default(),
            Ok(Vec::new()),
            false,
            CrateOrigin::CratesIo {
                repo: None,
                name: display_name.map(|n| n.canonical_name().to_string()),
            },
            target_layout.clone(),
        );

        public_deps.add_to_crate_graph(&mut crate_graph, detached_file_crate);
    }
    crate_graph
}

fn handle_crablangc_crates(
    crate_graph: &mut CrateGraph,
    pkg_to_lib_crate: &mut FxHashMap<Package, CrateId>,
    load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
    load_proc_macro: &mut dyn FnMut(&str, &AbsPath) -> ProcMacroLoadResult,
    crablangc_workspace: &CargoWorkspace,
    cargo: &CargoWorkspace,
    public_deps: &SysrootPublicDeps,
    libproc_macro: Option<CrateId>,
    pkg_crates: &FxHashMap<Package, Vec<(CrateId, TargetKind)>>,
    cfg_options: &CfgOptions,
    override_cfg: &CfgOverrides,
    build_scripts: &WorkspaceBuildScripts,
    target_layout: TargetLayoutLoadResult,
) {
    let mut crablangc_pkg_crates = FxHashMap::default();
    // The root package of the crablangc-dev component is crablangc_driver, so we match that
    let root_pkg =
        crablangc_workspace.packages().find(|&package| crablangc_workspace[package].name == "crablangc_driver");
    // The crablangc workspace might be incomplete (such as if crablangc-dev is not
    // installed for the current toolchain) and `crablangc_source` is set to discover.
    if let Some(root_pkg) = root_pkg {
        // Iterate through every crate in the dependency subtree of crablangc_driver using BFS
        let mut queue = VecDeque::new();
        queue.push_back(root_pkg);
        while let Some(pkg) = queue.pop_front() {
            // Don't duplicate packages if they are dependent on a diamond pattern
            // N.B. if this line is omitted, we try to analyze over 4_800_000 crates
            // which is not ideal
            if crablangc_pkg_crates.contains_key(&pkg) {
                continue;
            }
            for dep in &crablangc_workspace[pkg].dependencies {
                queue.push_back(dep.pkg);
            }

            let mut cfg_options = cfg_options.clone();

            let overrides = match override_cfg {
                CfgOverrides::Wildcard(cfg_diff) => Some(cfg_diff),
                CfgOverrides::Selective(cfg_overrides) => {
                    cfg_overrides.get(&crablangc_workspace[pkg].name)
                }
            };

            if let Some(overrides) = overrides {
                // FIXME: this is sort of a hack to deal with #![cfg(not(test))] vanishing such as seen
                // in ed25519_dalek (#7243), and libcore (#9203) (although you only hit that one while
                // working on crablang/crablang as that's the only time it appears outside sysroot).
                //
                // A more ideal solution might be to reanalyze crates based on where the cursor is and
                // figure out the set of cfgs that would have to apply to make it active.

                cfg_options.apply_diff(overrides.clone());
            };

            for &tgt in crablangc_workspace[pkg].targets.iter() {
                if crablangc_workspace[tgt].kind != TargetKind::Lib {
                    continue;
                }
                if let Some(file_id) = load(&crablangc_workspace[tgt].root) {
                    let crate_id = add_target_crate_root(
                        crate_graph,
                        &crablangc_workspace[pkg],
                        build_scripts.get_output(pkg),
                        cfg_options.clone(),
                        &mut |path| load_proc_macro(&crablangc_workspace[tgt].name, path),
                        file_id,
                        &crablangc_workspace[tgt].name,
                        crablangc_workspace[tgt].is_proc_macro,
                        target_layout.clone(),
                    );
                    pkg_to_lib_crate.insert(pkg, crate_id);
                    // Add dependencies on core / std / alloc for this crate
                    public_deps.add_to_crate_graph(crate_graph, crate_id);
                    if let Some(proc_macro) = libproc_macro {
                        add_dep_with_prelude(
                            crate_graph,
                            crate_id,
                            CrateName::new("proc_macro").unwrap(),
                            proc_macro,
                            crablangc_workspace[tgt].is_proc_macro,
                        );
                    }
                    crablangc_pkg_crates.entry(pkg).or_insert_with(Vec::new).push(crate_id);
                }
            }
        }
    }
    // Now add a dep edge from all targets of upstream to the lib
    // target of downstream.
    for pkg in crablangc_pkg_crates.keys().copied() {
        for dep in crablangc_workspace[pkg].dependencies.iter() {
            let name = CrateName::new(&dep.name).unwrap();
            if let Some(&to) = pkg_to_lib_crate.get(&dep.pkg) {
                for &from in crablangc_pkg_crates.get(&pkg).into_iter().flatten() {
                    add_dep(crate_graph, from, name.clone(), to);
                }
            }
        }
    }
    // Add a dependency on the crablangc_private crates for all targets of each package
    // which opts in
    for dep in crablangc_workspace.packages() {
        let name = CrateName::normalize_dashes(&crablangc_workspace[dep].name);

        if let Some(&to) = pkg_to_lib_crate.get(&dep) {
            for pkg in cargo.packages() {
                let package = &cargo[pkg];
                if !package.metadata.crablangc_private {
                    continue;
                }
                for (from, _) in pkg_crates.get(&pkg).into_iter().flatten() {
                    // Avoid creating duplicate dependencies
                    // This avoids the situation where `from` depends on e.g. `arrayvec`, but
                    // `crablang_analyzer` thinks that it should use the one from the `crablangc_source`
                    // instead of the one from `crates.io`
                    if !crate_graph[*from].dependencies.iter().any(|d| d.name == name) {
                        add_dep(crate_graph, *from, name.clone(), to);
                    }
                }
            }
        }
    }
}

fn add_target_crate_root(
    crate_graph: &mut CrateGraph,
    pkg: &PackageData,
    build_data: Option<&BuildScriptOutput>,
    cfg_options: CfgOptions,
    load_proc_macro: &mut dyn FnMut(&AbsPath) -> ProcMacroLoadResult,
    file_id: FileId,
    cargo_name: &str,
    is_proc_macro: bool,
    target_layout: TargetLayoutLoadResult,
) -> CrateId {
    let edition = pkg.edition;
    let mut potential_cfg_options = cfg_options.clone();
    potential_cfg_options.extend(
        pkg.features
            .iter()
            .map(|feat| CfgFlag::KeyValue { key: "feature".into(), value: feat.0.into() }),
    );
    let cfg_options = {
        let mut opts = cfg_options;
        for feature in pkg.active_features.iter() {
            opts.insert_key_value("feature".into(), feature.into());
        }
        if let Some(cfgs) = build_data.as_ref().map(|it| &it.cfgs) {
            opts.extend(cfgs.iter().cloned());
        }
        opts
    };

    let mut env = Env::default();
    inject_cargo_env(pkg, &mut env);

    if let Some(envs) = build_data.map(|it| &it.envs) {
        for (k, v) in envs {
            env.set(k, v.clone());
        }
    }

    let proc_macro = match build_data.as_ref().map(|it| it.proc_macro_dylib_path.as_ref()) {
        Some(Some(it)) => load_proc_macro(it),
        Some(None) => Err("no proc macro dylib present".into()),
        None => Err("crate has not (yet) been built".into()),
    };

    let display_name = CrateDisplayName::from_canonical_name(cargo_name.to_string());
    crate_graph.add_crate_root(
        file_id,
        edition,
        Some(display_name),
        Some(pkg.version.to_string()),
        cfg_options,
        potential_cfg_options,
        env,
        proc_macro,
        is_proc_macro,
        CrateOrigin::CratesIo { repo: pkg.repository.clone(), name: Some(pkg.name.clone()) },
        target_layout,
    )
}

#[derive(Default)]
struct SysrootPublicDeps {
    deps: Vec<(CrateName, CrateId, bool)>,
}

impl SysrootPublicDeps {
    /// Makes `from` depend on the public sysroot crates.
    fn add_to_crate_graph(&self, crate_graph: &mut CrateGraph, from: CrateId) {
        for (name, krate, prelude) in &self.deps {
            add_dep_with_prelude(crate_graph, from, name.clone(), *krate, *prelude);
        }
    }
}

fn sysroot_to_crate_graph(
    crate_graph: &mut CrateGraph,
    sysroot: &Sysroot,
    crablangc_cfg: Vec<CfgFlag>,
    target_layout: TargetLayoutLoadResult,
    load: &mut dyn FnMut(&AbsPath) -> Option<FileId>,
) -> (SysrootPublicDeps, Option<CrateId>) {
    let _p = profile::span("sysroot_to_crate_graph");
    let mut cfg_options = CfgOptions::default();
    cfg_options.extend(crablangc_cfg);
    let sysroot_crates: FxHashMap<SysrootCrate, CrateId> = sysroot
        .crates()
        .filter_map(|krate| {
            let file_id = load(&sysroot[krate].root)?;

            let env = Env::default();
            let display_name = CrateDisplayName::from_canonical_name(sysroot[krate].name.clone());
            let crate_id = crate_graph.add_crate_root(
                file_id,
                Edition::CURRENT,
                Some(display_name),
                None,
                cfg_options.clone(),
                cfg_options.clone(),
                env,
                Err("no proc macro loaded for sysroot crate".into()),
                false,
                CrateOrigin::Lang(LangCrateOrigin::from(&*sysroot[krate].name)),
                target_layout.clone(),
            );
            Some((krate, crate_id))
        })
        .collect();

    for from in sysroot.crates() {
        for &to in sysroot[from].deps.iter() {
            let name = CrateName::new(&sysroot[to].name).unwrap();
            if let (Some(&from), Some(&to)) = (sysroot_crates.get(&from), sysroot_crates.get(&to)) {
                add_dep(crate_graph, from, name, to);
            }
        }
    }

    let public_deps = SysrootPublicDeps {
        deps: sysroot
            .public_deps()
            .map(|(name, idx, prelude)| (name, sysroot_crates[&idx], prelude))
            .collect::<Vec<_>>(),
    };

    let libproc_macro = sysroot.proc_macro().and_then(|it| sysroot_crates.get(&it).copied());
    (public_deps, libproc_macro)
}

fn add_dep(graph: &mut CrateGraph, from: CrateId, name: CrateName, to: CrateId) {
    add_dep_inner(graph, from, Dependency::new(name, to))
}

fn add_dep_with_prelude(
    graph: &mut CrateGraph,
    from: CrateId,
    name: CrateName,
    to: CrateId,
    prelude: bool,
) {
    add_dep_inner(graph, from, Dependency::with_prelude(name, to, prelude))
}

fn add_dep_inner(graph: &mut CrateGraph, from: CrateId, dep: Dependency) {
    if let Err(err) = graph.add_dep(from, dep) {
        tracing::error!("{}", err)
    }
}

/// Recreates the compile-time environment variables that Cargo sets.
///
/// Should be synced with
/// <https://doc.crablang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates>
///
/// FIXME: ask Cargo to provide this data instead of re-deriving.
fn inject_cargo_env(package: &PackageData, env: &mut Env) {
    // FIXME: Missing variables:
    // CARGO_BIN_NAME, CARGO_BIN_EXE_<name>

    let manifest_dir = package.manifest.parent();
    env.set("CARGO_MANIFEST_DIR", manifest_dir.as_os_str().to_string_lossy().into_owned());

    // Not always right, but works for common cases.
    env.set("CARGO", "cargo".into());

    env.set("CARGO_PKG_VERSION", package.version.to_string());
    env.set("CARGO_PKG_VERSION_MAJOR", package.version.major.to_string());
    env.set("CARGO_PKG_VERSION_MINOR", package.version.minor.to_string());
    env.set("CARGO_PKG_VERSION_PATCH", package.version.patch.to_string());
    env.set("CARGO_PKG_VERSION_PRE", package.version.pre.to_string());

    env.set("CARGO_PKG_AUTHORS", String::new());

    env.set("CARGO_PKG_NAME", package.name.clone());
    // FIXME: This isn't really correct (a package can have many crates with different names), but
    // it's better than leaving the variable unset.
    env.set("CARGO_CRATE_NAME", CrateName::normalize_dashes(&package.name).to_string());
    env.set("CARGO_PKG_DESCRIPTION", String::new());
    env.set("CARGO_PKG_HOMEPAGE", String::new());
    env.set("CARGO_PKG_REPOSITORY", String::new());
    env.set("CARGO_PKG_LICENSE", String::new());

    env.set("CARGO_PKG_LICENSE_FILE", String::new());
}
