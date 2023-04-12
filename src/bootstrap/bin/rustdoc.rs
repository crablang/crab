//! Shim which is passed to Cargo as "crablangdoc" when running the bootstrap.
//!
//! See comments in `src/bootstrap/crablangc.rs` for more information.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

include!("../dylib_util.rs");

fn main() {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let stage = env::var("CRABLANGC_STAGE").expect("CRABLANGC_STAGE was not set");
    let crablangdoc = env::var_os("CRABLANGDOC_REAL").expect("CRABLANGDOC_REAL was not set");
    let libdir = env::var_os("CRABLANGDOC_LIBDIR").expect("CRABLANGDOC_LIBDIR was not set");
    let sysroot = env::var_os("CRABLANGC_SYSROOT").expect("CRABLANGC_SYSROOT was not set");

    // Detect whether or not we're a build script depending on whether --target
    // is passed (a bit janky...)
    let target = args.windows(2).find(|w| &*w[0] == "--target").and_then(|w| w[1].to_str());

    use std::str::FromStr;

    let verbose = match env::var("CRABLANGC_VERBOSE") {
        Ok(s) => usize::from_str(&s).expect("CRABLANGC_VERBOSE should be an integer"),
        Err(_) => 0,
    };

    let mut dylib_path = dylib_path();
    dylib_path.insert(0, PathBuf::from(libdir.clone()));

    let mut cmd = Command::new(crablangdoc);

    if target.is_some() {
        // The stage0 compiler has a special sysroot distinct from what we
        // actually downloaded, so we just always pass the `--sysroot` option,
        // unless one is already set.
        if !args.iter().any(|arg| arg == "--sysroot") {
            cmd.arg("--sysroot").arg(&sysroot);
        }
    }

    cmd.args(&args);
    cmd.env(dylib_path_var(), env::join_paths(&dylib_path).unwrap());

    // Force all crates compiled by this compiler to (a) be unstable and (b)
    // allow the `crablangc_private` feature to link to other unstable crates
    // also in the sysroot.
    if env::var_os("CRABLANGC_FORCE_UNSTABLE").is_some() {
        cmd.arg("-Z").arg("force-unstable-if-unmarked");
    }
    if let Some(linker) = env::var_os("CRABLANGDOC_LINKER") {
        let mut arg = OsString::from("-Clinker=");
        arg.push(&linker);
        cmd.arg(arg);
    }
    if let Ok(no_threads) = env::var("CRABLANGDOC_LLD_NO_THREADS") {
        cmd.arg("-Clink-arg=-fuse-ld=lld");
        cmd.arg(format!("-Clink-arg=-Wl,{}", no_threads));
    }
    // Cargo doesn't pass CRABLANGDOCFLAGS to proc_macros:
    // https://github.com/crablang/cargo/issues/4423
    // Thus, if we are on stage 0, we explicitly set `--cfg=bootstrap`.
    // We also declare that the flag is expected, which we need to do to not
    // get warnings about it being unexpected.
    if stage == "0" {
        cmd.arg("--cfg=bootstrap");
    }
    cmd.arg("-Zunstable-options");
    cmd.arg("--check-cfg=values(bootstrap)");

    if verbose > 1 {
        eprintln!(
            "crablangdoc command: {:?}={:?} {:?}",
            dylib_path_var(),
            env::join_paths(&dylib_path).unwrap(),
            cmd,
        );
        eprintln!("sysroot: {:?}", sysroot);
        eprintln!("libdir: {:?}", libdir);
    }

    std::process::exit(match cmd.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => panic!("\n\nfailed to run {:?}: {}\n\n", cmd, e),
    })
}
