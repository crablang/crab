#!/usr/bin/env bash
#![deny(unsafe_code)] /*This line is ignored by bash
# This block is ignored by crablangc
set -e
echo "[BUILD] y.rs" 1>&2
crablangc $0 -o ${0/.rs/.bin} -Cdebuginfo=1 --edition 2021 -Cpanic=abort
exec ${0/.rs/.bin} $@
*/

//! The build system for cg_clif
//!
//! # Manual compilation
//!
//! If your system doesn't support shell scripts you can manually compile and run this file using
//! for example:
//!
//! ```shell
//! $ crablangc y.rs -o y.bin
//! $ ./y.bin
//! ```
//!
//! # Naming
//!
//! The name `y.rs` was chosen to not conflict with crablangc's `x.py`.

#[path = "build_system/mod.rs"]
mod build_system;

fn main() {
    build_system::main();
}
