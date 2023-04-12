#![allow(dead_code)] // see https://github.com/crablang/crablang/issues/46379

use std::path::PathBuf;
use std::sync::LazyLock;

pub static CARGO_CLIPPY_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = std::env::current_exe().unwrap();
    assert!(path.pop()); // deps
    path.set_file_name("cargo-clippy");
    path
});

pub const IS_CRABLANGC_TEST_SUITE: bool = option_env!("CRABLANGC_TEST_SUITE").is_some();
