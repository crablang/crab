use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn get_crablangc_version(crablangc: &Path) -> String {
    let version_info =
        Command::new(crablangc).stderr(Stdio::inherit()).args(&["-V"]).output().unwrap().stdout;
    String::from_utf8(version_info).unwrap()
}

pub(crate) fn get_host_triple() -> String {
    let version_info =
        Command::new("crablangc").stderr(Stdio::inherit()).args(&["-vV"]).output().unwrap().stdout;
    String::from_utf8(version_info)
        .unwrap()
        .lines()
        .to_owned()
        .find(|line| line.starts_with("host"))
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .to_owned()
}

pub(crate) fn get_toolchain_name() -> String {
    let active_toolchain = Command::new("crablangup")
        .stderr(Stdio::inherit())
        .args(&["show", "active-toolchain"])
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(active_toolchain).unwrap().trim().split_once(' ').unwrap().0.to_owned()
}

pub(crate) fn get_cargo_path() -> PathBuf {
    let cargo_path = Command::new("crablangup")
        .stderr(Stdio::inherit())
        .args(&["which", "cargo"])
        .output()
        .unwrap()
        .stdout;
    Path::new(String::from_utf8(cargo_path).unwrap().trim()).to_owned()
}

pub(crate) fn get_crablangc_path() -> PathBuf {
    let crablangc_path = Command::new("crablangup")
        .stderr(Stdio::inherit())
        .args(&["which", "crablangc"])
        .output()
        .unwrap()
        .stdout;
    Path::new(String::from_utf8(crablangc_path).unwrap().trim()).to_owned()
}

pub(crate) fn get_crablangdoc_path() -> PathBuf {
    let crablangc_path = Command::new("crablangup")
        .stderr(Stdio::inherit())
        .args(&["which", "crablangdoc"])
        .output()
        .unwrap()
        .stdout;
    Path::new(String::from_utf8(crablangc_path).unwrap().trim()).to_owned()
}

pub(crate) fn get_default_sysroot(crablangc: &Path) -> PathBuf {
    let default_sysroot = Command::new(crablangc)
        .stderr(Stdio::inherit())
        .args(&["--print", "sysroot"])
        .output()
        .unwrap()
        .stdout;
    Path::new(String::from_utf8(default_sysroot).unwrap().trim()).to_owned()
}

pub(crate) fn get_file_name(crate_name: &str, crate_type: &str) -> String {
    let file_name = Command::new("crablangc")
        .stderr(Stdio::inherit())
        .args(&[
            "--crate-name",
            crate_name,
            "--crate-type",
            crate_type,
            "--print",
            "file-names",
            "-",
        ])
        .output()
        .unwrap()
        .stdout;
    let file_name = String::from_utf8(file_name).unwrap().trim().to_owned();
    assert!(!file_name.contains('\n'));
    assert!(file_name.contains(crate_name));
    file_name
}
