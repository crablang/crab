//! Determine crablangc version `proc-macro-srv` (and thus the sysroot ABI) is
//! build with and make it accessible at runtime for ABI selection.

use std::{env, fs::File, io::Write, path::PathBuf, process::Command};

fn main() {
    let mut path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    path.push("crablangc_version.rs");
    let mut f = File::create(&path).unwrap();

    let crablangc = env::var("CRABLANGC").expect("proc-macro-srv's build script expects CRABLANGC to be set");
    let output = Command::new(crablangc).arg("--version").output().expect("crablangc --version must run");
    let version_string = std::str::from_utf8(&output.stdout[..])
        .expect("crablangc --version output must be UTF-8")
        .trim();

    write!(
        f,
        "
    #[allow(dead_code)]
    pub(crate) const CRABLANGC_VERSION_STRING: &str = {version_string:?};
    "
    )
    .unwrap();
}
