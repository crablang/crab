//! Some stuff used by crablangc that doesn't have many dependencies
//!
//! Originally extracted from crablangc::back, which was nominally the
//! compiler 'backend', though LLVM is crablangc's backend, so crablangc_target
//! is really just odds-and-ends relating to code gen and linking.
//! This crate mostly exists to make crablangc smaller, so we might put
//! more 'stuff' here in the future. It does not have a dependency on
//! LLVM.

#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(assert_matches)]
#![feature(associated_type_bounds)]
#![feature(exhaustive_patterns)]
#![feature(min_specialization)]
#![feature(never_type)]
#![feature(crablangc_attrs)]
#![feature(step_trait)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

use std::path::{Path, PathBuf};

#[macro_use]
extern crate crablangc_macros;

#[macro_use]
extern crate tracing;

pub mod abi;
pub mod asm;
pub mod json;
pub mod spec;

#[cfg(test)]
mod tests;

pub use crablangc_abi::HashStableContext;

/// The name of crablangc's own place to organize libraries.
///
/// Used to be `crablangc`, now the default is `crablanglib`.
const CRABLANG_LIB_DIR: &str = "crablanglib";

/// Returns a `crablanglib` path for this particular target, relative to the provided sysroot.
///
/// For example: `target_sysroot_path("/usr", "x86_64-unknown-linux-gnu")` =>
/// `"lib*/crablanglib/x86_64-unknown-linux-gnu"`.
pub fn target_crablanglib_path(sysroot: &Path, target_triple: &str) -> PathBuf {
    let libdir = find_libdir(sysroot);
    PathBuf::from_iter([
        Path::new(libdir.as_ref()),
        Path::new(CRABLANG_LIB_DIR),
        Path::new(target_triple),
    ])
}

/// The name of the directory crablangc expects libraries to be located.
fn find_libdir(sysroot: &Path) -> std::borrow::Cow<'static, str> {
    // FIXME: This is a quick hack to make the crablangc binary able to locate
    // CrabLang libraries in Linux environments where libraries might be installed
    // to lib64/lib32. This would be more foolproof by basing the sysroot off
    // of the directory where `libcrablangc_driver` is located, rather than
    // where the crablangc binary is.
    // If --libdir is set during configuration to the value other than
    // "lib" (i.e., non-default), this value is used (see issue #16552).

    #[cfg(target_pointer_width = "64")]
    const PRIMARY_LIB_DIR: &str = "lib64";

    #[cfg(target_pointer_width = "32")]
    const PRIMARY_LIB_DIR: &str = "lib32";

    const SECONDARY_LIB_DIR: &str = "lib";

    match option_env!("CFG_LIBDIR_RELATIVE") {
        None | Some("lib") => {
            if sysroot.join(PRIMARY_LIB_DIR).join(CRABLANG_LIB_DIR).exists() {
                PRIMARY_LIB_DIR.into()
            } else {
                SECONDARY_LIB_DIR.into()
            }
        }
        Some(libdir) => libdir.into(),
    }
}
