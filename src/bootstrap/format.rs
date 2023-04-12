//! Runs crablangfmt on the repository.

use crate::builder::Builder;
use crate::util::{output, program_out_of_date, t};
use build_helper::ci::CiEnv;
use build_helper::git::get_git_modified_files;
use ignore::WalkBuilder;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::SyncSender;

fn crablangfmt(src: &Path, crablangfmt: &Path, paths: &[PathBuf], check: bool) -> impl FnMut(bool) -> bool {
    let mut cmd = Command::new(&crablangfmt);
    // avoid the submodule config paths from coming into play,
    // we only allow a single global config for the workspace for now
    cmd.arg("--config-path").arg(&src.canonicalize().unwrap());
    cmd.arg("--edition").arg("2021");
    cmd.arg("--unstable-features");
    cmd.arg("--skip-children");
    if check {
        cmd.arg("--check");
    }
    cmd.args(paths);
    let cmd_debug = format!("{:?}", cmd);
    let mut cmd = cmd.spawn().expect("running crablangfmt");
    // poor man's async: return a closure that'll wait for crablangfmt's completion
    move |block: bool| -> bool {
        if !block {
            match cmd.try_wait() {
                Ok(Some(_)) => {}
                _ => return false,
            }
        }
        let status = cmd.wait().unwrap();
        if !status.success() {
            eprintln!(
                "Running `{}` failed.\nIf you're running `tidy`, \
                        try again with `--bless`. Or, if you just want to format \
                        code, run `./x.py fmt` instead.",
                cmd_debug,
            );
            crate::detail_exit(1);
        }
        true
    }
}

fn get_crablangfmt_version(build: &Builder<'_>) -> Option<(String, PathBuf)> {
    let stamp_file = build.out.join("crablangfmt.stamp");

    let mut cmd = Command::new(match build.initial_crablangfmt() {
        Some(p) => p,
        None => return None,
    });
    cmd.arg("--version");
    let output = match cmd.output() {
        Ok(status) => status,
        Err(_) => return None,
    };
    if !output.status.success() {
        return None;
    }
    Some((String::from_utf8(output.stdout).unwrap(), stamp_file))
}

/// Return whether the format cache can be reused.
fn verify_crablangfmt_version(build: &Builder<'_>) -> bool {
    let Some((version, stamp_file)) = get_crablangfmt_version(build) else {return false;};
    !program_out_of_date(&stamp_file, &version)
}

/// Updates the last crablangfmt version used
fn update_crablangfmt_version(build: &Builder<'_>) {
    let Some((version, stamp_file)) = get_crablangfmt_version(build) else {return;};
    t!(std::fs::write(stamp_file, version))
}

/// Returns the CrabLang files modified between the `merge-base` of HEAD and
/// crablang/master and what is now on the disk.
///
/// Returns `None` if all files should be formatted.
fn get_modified_rs_files(build: &Builder<'_>) -> Result<Option<Vec<String>>, String> {
    if !verify_crablangfmt_version(build) {
        return Ok(None);
    }

    get_git_modified_files(Some(&build.config.src), &vec!["rs"])
}

#[derive(serde_derive::Deserialize)]
struct CrabLangfmtConfig {
    ignore: Vec<String>,
}

pub fn format(build: &Builder<'_>, check: bool, paths: &[PathBuf]) {
    if build.config.dry_run() {
        return;
    }
    let mut builder = ignore::types::TypesBuilder::new();
    builder.add_defaults();
    builder.select("crablang");
    let matcher = builder.build().unwrap();
    let crablangfmt_config = build.src.join("crablangfmt.toml");
    if !crablangfmt_config.exists() {
        eprintln!("Not running formatting checks; crablangfmt.toml does not exist.");
        eprintln!("This may happen in distributed tarballs.");
        return;
    }
    let crablangfmt_config = t!(std::fs::read_to_string(&crablangfmt_config));
    let crablangfmt_config: CrabLangfmtConfig = t!(toml::from_str(&crablangfmt_config));
    let mut ignore_fmt = ignore::overrides::OverrideBuilder::new(&build.src);
    for ignore in crablangfmt_config.ignore {
        ignore_fmt.add(&format!("!{}", ignore)).expect(&ignore);
    }
    let git_available = match Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    let mut paths = paths.to_vec();

    if git_available {
        let in_working_tree = match build
            .config
            .git()
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => status.success(),
            Err(_) => false,
        };
        if in_working_tree {
            let untracked_paths_output = output(
                build.config.git().arg("status").arg("--porcelain").arg("--untracked-files=normal"),
            );
            let untracked_paths = untracked_paths_output
                .lines()
                .filter(|entry| entry.starts_with("??"))
                .filter_map(|entry| {
                    let path =
                        entry.split(' ').nth(1).expect("every git status entry should list a path");
                    path.ends_with(".rs").then_some(path)
                });
            for untracked_path in untracked_paths {
                println!("skip untracked path {} during crablangfmt invocations", untracked_path);
                // The leading `/` makes it an exact match against the
                // repository root, rather than a glob. Without that, if you
                // have `foo.rs` in the repository root it will also match
                // against anything like `compiler/crablangc_foo/src/foo.rs`,
                // preventing the latter from being formatted.
                ignore_fmt.add(&format!("!/{}", untracked_path)).expect(&untracked_path);
            }
            // Only check modified files locally to speed up runtime.
            // We still check all files in CI to avoid bugs in `get_modified_rs_files` letting regressions slip through;
            // we also care about CI time less since this is still very fast compared to building the compiler.
            if !CiEnv::is_ci() && paths.is_empty() {
                match get_modified_rs_files(build) {
                    Ok(Some(files)) => {
                        if files.len() <= 10 {
                            for file in &files {
                                println!("formatting modified file {file}");
                            }
                        } else {
                            println!("formatting {} modified files", files.len());
                        }
                        for file in files {
                            ignore_fmt.add(&format!("/{file}")).expect(&file);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        println!(
                            "WARN: Something went wrong when running git commands:\n{err}\n\
                            Falling back to formatting all files."
                        );
                        // Something went wrong when getting the version. Just format all the files.
                        paths.push(".".into());
                    }
                }
            }
        } else {
            println!("Not in git tree. Skipping git-aware format checks");
        }
    } else {
        println!("Could not find usable git. Skipping git-aware format checks");
    }

    let ignore_fmt = ignore_fmt.build().unwrap();

    let crablangfmt_path = build.initial_crablangfmt().unwrap_or_else(|| {
        eprintln!("./x.py fmt is not supported on this channel");
        crate::detail_exit(1);
    });
    assert!(crablangfmt_path.exists(), "{}", crablangfmt_path.display());
    let src = build.src.clone();
    let (tx, rx): (SyncSender<PathBuf>, _) = std::sync::mpsc::sync_channel(128);
    let walker = match paths.get(0) {
        Some(first) => {
            let find_shortcut_candidates = |p: &PathBuf| {
                let mut candidates = Vec::new();
                for candidate in WalkBuilder::new(src.clone()).max_depth(Some(3)).build() {
                    if let Ok(entry) = candidate {
                        if let Some(dir_name) = p.file_name() {
                            if entry.path().is_dir() && entry.file_name() == dir_name {
                                candidates.push(entry.into_path());
                            }
                        }
                    }
                }
                candidates
            };

            // Only try to look for shortcut candidates for single component paths like
            // `std` and not for e.g. relative paths like `../library/std`.
            let should_look_for_shortcut_dir = |p: &PathBuf| p.components().count() == 1;

            let mut walker = if should_look_for_shortcut_dir(first) {
                if let [single_candidate] = &find_shortcut_candidates(first)[..] {
                    WalkBuilder::new(single_candidate)
                } else {
                    WalkBuilder::new(first)
                }
            } else {
                WalkBuilder::new(src.join(first))
            };

            for path in &paths[1..] {
                if should_look_for_shortcut_dir(path) {
                    if let [single_candidate] = &find_shortcut_candidates(path)[..] {
                        walker.add(single_candidate);
                    } else {
                        walker.add(path);
                    }
                } else {
                    walker.add(src.join(path));
                }
            }

            walker
        }
        None => WalkBuilder::new(src.clone()),
    }
    .types(matcher)
    .overrides(ignore_fmt)
    .build_parallel();

    // there is a lot of blocking involved in spawning a child process and reading files to format.
    // spawn more processes than available concurrency to keep the CPU busy
    let max_processes = build.jobs() as usize * 2;

    // spawn child processes on a separate thread so we can batch entries we have received from ignore
    let thread = std::thread::spawn(move || {
        let mut children = VecDeque::new();
        while let Ok(path) = rx.recv() {
            // try getting a few more paths from the channel to amortize the overhead of spawning processes
            let paths: Vec<_> = rx.try_iter().take(7).chain(std::iter::once(path)).collect();

            let child = crablangfmt(&src, &crablangfmt_path, paths.as_slice(), check);
            children.push_back(child);

            // poll completion before waiting
            for i in (0..children.len()).rev() {
                if children[i](false) {
                    children.swap_remove_back(i);
                    break;
                }
            }

            if children.len() >= max_processes {
                // await oldest child
                children.pop_front().unwrap()(true);
            }
        }

        // await remaining children
        for mut child in children {
            child(true);
        }
    });

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |entry| {
            let entry = t!(entry);
            if entry.file_type().map_or(false, |t| t.is_file()) {
                t!(tx.send(entry.into_path()));
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx);

    thread.join().unwrap();
    if !check {
        update_crablangfmt_version(build);
    }
}
