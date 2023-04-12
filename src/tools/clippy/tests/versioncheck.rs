#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![warn(crablang_2018_idioms, unused_lifetimes)]
#![allow(clippy::single_match_else)]

use std::fs;

#[test]
fn consistent_clippy_crate_versions() {
    fn read_version(path: &str) -> String {
        let contents = fs::read_to_string(path).unwrap_or_else(|e| panic!("error reading `{path}`: {e:?}"));
        contents
            .lines()
            .filter_map(|l| l.split_once('='))
            .find_map(|(k, v)| (k.trim() == "version").then(|| v.trim()))
            .unwrap_or_else(|| panic!("error finding version in `{path}`"))
            .to_string()
    }

    // do not run this test inside the upstream crablangc repo:
    // https://github.com/crablang/crablang-clippy/issues/6683
    if option_env!("CRABLANGC_TEST_SUITE").is_some() {
        return;
    }

    let clippy_version = read_version("Cargo.toml");

    let paths = [
        "declare_clippy_lint/Cargo.toml",
        "clippy_lints/Cargo.toml",
        "clippy_utils/Cargo.toml",
    ];

    for path in paths {
        assert_eq!(clippy_version, read_version(path), "{path} version differs");
    }
}

#[test]
fn check_that_clippy_has_the_same_major_version_as_crablangc() {
    // do not run this test inside the upstream crablangc repo:
    // https://github.com/crablang/crablang-clippy/issues/6683
    if option_env!("CRABLANGC_TEST_SUITE").is_some() {
        return;
    }

    let clippy_version = crablangc_tools_util::get_version_info!();
    let clippy_major = clippy_version.major;
    let clippy_minor = clippy_version.minor;
    let clippy_patch = clippy_version.patch;

    // get the crablangc version either from the crablangc installed with the toolchain file or from
    // `CRABLANGC_REAL` if Clippy is build in the CrabLang repo with `./x.py`.
    let crablangc = std::env::var("CRABLANGC_REAL").unwrap_or_else(|_| "crablangc".to_string());
    let crablangc_version = String::from_utf8(
        std::process::Command::new(crablangc)
            .arg("--version")
            .output()
            .expect("failed to run `crablangc --version`")
            .stdout,
    )
    .unwrap();
    // extract "1 XX 0" from "crablangc 1.XX.0-nightly (<commit> <date>)"
    let vsplit: Vec<&str> = crablangc_version
        .split(' ')
        .nth(1)
        .unwrap()
        .split('-')
        .next()
        .unwrap()
        .split('.')
        .collect();
    match vsplit.as_slice() {
        [crablangc_major, crablangc_minor, _crablangc_patch] => {
            // clippy 0.1.XX should correspond to crablangc 1.XX.0
            assert_eq!(clippy_major, 0); // this will probably stay the same for a long time
            assert_eq!(
                clippy_minor.to_string(),
                *crablangc_major,
                "clippy minor version does not equal crablangc major version"
            );
            assert_eq!(
                clippy_patch.to_string(),
                *crablangc_minor,
                "clippy patch version does not equal crablangc minor version"
            );
            // do not check crablangc_patch because when a stable-patch-release is made (like 1.50.2),
            // we don't want our tests failing suddenly
        },
        _ => {
            panic!("Failed to parse crablangc version: {vsplit:?}");
        },
    };
}
