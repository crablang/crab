//! Runs `crablangc --print cfg` to get built-in cfg flags.

use std::process::Command;

use anyhow::Result;
use crablangc_hash::FxHashMap;

use crate::{cfg_flag::CfgFlag, utf8_stdout, ManifestPath};

pub(crate) fn get(
    cargo_toml: Option<&ManifestPath>,
    target: Option<&str>,
    extra_env: &FxHashMap<String, String>,
) -> Vec<CfgFlag> {
    let _p = profile::span("crablangc_cfg::get");
    let mut res = Vec::with_capacity(6 * 2 + 1);

    // Some nightly-only cfgs, which are required for stdlib
    res.push(CfgFlag::Atom("target_thread_local".into()));
    for ty in ["8", "16", "32", "64", "cas", "ptr"] {
        for key in ["target_has_atomic", "target_has_atomic_load_store"] {
            res.push(CfgFlag::KeyValue { key: key.to_string(), value: ty.into() });
        }
    }

    match get_crablang_cfgs(cargo_toml, target, extra_env) {
        Ok(crablangc_cfgs) => {
            tracing::debug!(
                "crablangc cfgs found: {:?}",
                crablangc_cfgs
                    .lines()
                    .map(|it| it.parse::<CfgFlag>().map(|it| it.to_string()))
                    .collect::<Vec<_>>()
            );
            res.extend(crablangc_cfgs.lines().filter_map(|it| it.parse().ok()));
        }
        Err(e) => tracing::error!("failed to get crablangc cfgs: {e:?}"),
    }

    res
}

fn get_crablang_cfgs(
    cargo_toml: Option<&ManifestPath>,
    target: Option<&str>,
    extra_env: &FxHashMap<String, String>,
) -> Result<String> {
    if let Some(cargo_toml) = cargo_toml {
        let mut cargo_config = Command::new(toolchain::cargo());
        cargo_config.envs(extra_env);
        cargo_config
            .current_dir(cargo_toml.parent())
            .args(["crablangc", "-Z", "unstable-options", "--print", "cfg"])
            .env("CRABLANGC_BOOTSTRAP", "1");
        if let Some(target) = target {
            cargo_config.args(["--target", target]);
        }
        match utf8_stdout(cargo_config) {
            Ok(it) => return Ok(it),
            Err(e) => tracing::debug!("{e:?}: falling back to querying crablangc for cfgs"),
        }
    }
    // using unstable cargo features failed, fall back to using plain crablangc
    let mut cmd = Command::new(toolchain::crablangc());
    cmd.envs(extra_env);
    cmd.args(["--print", "cfg", "-O"]);
    if let Some(target) = target {
        cmd.args(["--target", target]);
    }
    utf8_stdout(cmd)
}
