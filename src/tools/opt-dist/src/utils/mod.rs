pub mod io;

use crate::environment::Environment;
use crate::utils::io::{delete_directory, get_files_from_dir};
use humansize::{format_size, BINARY};
use sysinfo::{DiskExt, RefreshKind, System, SystemExt};

pub fn format_env_variables() -> String {
    let vars = std::env::vars().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>();
    vars.join("\n")
}

pub fn print_free_disk_space() -> anyhow::Result<()> {
    let sys = System::new_with_specifics(RefreshKind::default().with_disks_list().with_disks());
    let available_space: u64 = sys.disks().iter().map(|d| d.available_space()).sum();
    let total_space: u64 = sys.disks().iter().map(|d| d.total_space()).sum();
    let used_space = total_space - available_space;

    log::info!(
        "Free disk space: {} out of total {} ({:.2}% used)",
        humansize::format_size(available_space, BINARY),
        humansize::format_size(total_space, BINARY),
        (used_space as f64 / total_space as f64) * 100.0
    );
    Ok(())
}

pub fn print_binary_sizes(env: &dyn Environment) -> anyhow::Result<()> {
    use std::fmt::Write;

    let root = env.build_artifacts().join("stage2");

    let mut files = get_files_from_dir(&root.join("bin"), None)?;
    files.extend(get_files_from_dir(&root.join("lib"), Some(".so"))?);
    files.sort_unstable();

    let mut output = String::new();
    for file in files {
        let size = std::fs::metadata(file.as_std_path())?.len();
        let size_formatted = format_size(size, BINARY);
        let name = format!("{}:", file.file_name().unwrap());
        writeln!(output, "{name:<50}{size_formatted:>10}")?;
    }

    log::info!("Rustc artifact size\n{output}");

    Ok(())
}

pub fn clear_llvm_files(env: &dyn Environment) -> anyhow::Result<()> {
    // Bootstrap currently doesn't support rebuilding LLVM when PGO options
    // change (or any other llvm-related options); so just clear out the relevant
    // directories ourselves.
    log::info!("Clearing LLVM build files");
    delete_directory(&env.build_artifacts().join("llvm"))?;
    delete_directory(&env.build_artifacts().join("lld"))?;
    Ok(())
}

/// Wraps all output produced within the `func` closure in a CI output group, if we're running in
/// CI.
pub fn with_log_group<F: FnOnce() -> R, R>(group: &str, func: F) -> R {
    if is_in_ci() {
        println!("::group::{group}");
        let result = func();
        println!("::endgroup::");
        result
    } else {
        func()
    }
}

fn is_in_ci() -> bool {
    std::env::var("GITHUB_ACTIONS").is_ok()
}
