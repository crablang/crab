use crate::command_prelude::*;

use cargo::ops::{self, UpdateOptions};
use cargo::util::print_available_packages;

pub fn cli() -> Command {
    subcommand("update")
        .about("Update dependencies as recorded in the local lock file")
        .arg_quiet()
        .arg(flag("workspace", "Only update the workspace packages").short('w'))
        .arg_package_spec_simple("Package to update")
        .arg(flag(
            "aggressive",
            "Force updating all dependencies of SPEC as well when used with -p",
        ))
        .arg_dry_run("Don't actually write the lockfile")
        .arg(
            opt(
                "precise",
                "Update a single dependency to exactly PRECISE when used with -p",
            )
            .value_name("PRECISE")
            .requires("package"),
        )
        .arg_manifest_path()
        .after_help("Run `cargo help update` for more detailed information.\n")
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let ws = args.workspace(config)?;

    if args.is_present_with_zero_values("package") {
        print_available_packages(&ws)?;
    }

    let update_opts = UpdateOptions {
        aggressive: args.flag("aggressive"),
        precise: args.get_one::<String>("precise").map(String::as_str),
        to_update: values(args, "package"),
        dry_run: args.dry_run(),
        workspace: args.flag("workspace"),
        config,
    };
    ops::update_lockfile(&ws, &update_opts)?;
    Ok(())
}
