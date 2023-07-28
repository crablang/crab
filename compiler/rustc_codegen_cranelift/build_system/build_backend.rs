use std::env;
use std::path::PathBuf;

use super::path::{Dirs, RelPath};
use super::rustc_info::get_file_name;
use super::utils::{is_ci, is_ci_opt, maybe_incremental, CargoProject, Compiler, LogGroup};

pub(crate) static CG_CLIF: CargoProject = CargoProject::new(&RelPath::SOURCE, "cg_clif");

pub(crate) fn build_backend(
    dirs: &Dirs,
    channel: &str,
    bootstrap_host_compiler: &Compiler,
    use_unstable_features: bool,
) -> PathBuf {
    let _group = LogGroup::guard("Build backend");

    let mut cmd = CG_CLIF.build(&bootstrap_host_compiler, dirs);
    maybe_incremental(&mut cmd);

    let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();

    if is_ci() {
        // Deny warnings on CI
        rustflags += " -Dwarnings";

        if !is_ci_opt() {
            cmd.env("CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS", "true");
            cmd.env("CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS", "true");
        }
    }

    if use_unstable_features {
        cmd.arg("--features").arg("unstable-features");
    }

    match channel {
        "debug" => {}
        "release" => {
            cmd.arg("--release");
        }
        _ => unreachable!(),
    }

    cmd.env("RUSTFLAGS", rustflags);

    eprintln!("[BUILD] rustc_codegen_cranelift");
    super::utils::spawn_and_wait(cmd);

    CG_CLIF
        .target_dir(dirs)
        .join(&bootstrap_host_compiler.triple)
        .join(channel)
        .join(get_file_name(&bootstrap_host_compiler.rustc, "rustc_codegen_cranelift", "dylib"))
}
