use crate::command_prelude::*;

use cargo::ops;
use cargo::ops::FetchOptions;

pub fn cli() -> Command {
    subcommand("fetch")
        .about("Fetch dependencies of a package from the network")
        .arg_quiet()
        .arg_manifest_path()
        .arg_target_triple("Fetch dependencies for the target triple")
        .after_help("Run `cargo help fetch` for more detailed information.\n")
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let ws = args.workspace(config)?;

    let opts = FetchOptions {
        config,
        targets: args.targets(),
    };
    let _ = ops::fetch(&ws, &opts)?;
    Ok(())
}
