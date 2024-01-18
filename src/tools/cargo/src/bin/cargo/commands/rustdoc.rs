use cargo::ops::{self, DocOptions, OutputFormat};

use crate::command_prelude::*;

pub fn cli() -> Command {
    subcommand("rustdoc")
        .about("Build a package's documentation, using specified custom flags.")
        .arg(
            Arg::new("args")
                .value_name("ARGS")
                .help("Extra rustdoc flags")
                .num_args(0..)
                .trailing_var_arg(true),
        )
        .arg(flag(
            "open",
            "Opens the docs in a browser after the operation",
        ))
        .arg_ignore_rust_version()
        .arg_message_format()
        .arg_silent_suggestion()
        .arg_package("Package to document")
        .arg_targets_all(
            "Build only this package's library",
            "Build only the specified binary",
            "Build all binaries",
            "Build only the specified example",
            "Build all examples",
            "Build only the specified test target",
            "Build all test targets",
            "Build only the specified bench target",
            "Build all bench targets",
            "Build all targets",
        )
        .arg_features()
        .arg_parallel()
        .arg_release("Build artifacts in release mode, with optimizations")
        .arg_profile("Build artifacts with the specified profile")
        .arg_target_triple("Build for the target triple")
        .arg_target_dir()
        .arg(
            opt("output-format", "The output type to write (unstable)")
                .value_name("FMT")
                .value_parser(OutputFormat::POSSIBLE_VALUES),
        )
        .arg_unit_graph()
        .arg_timings()
        .arg_manifest_path()
        .after_help(color_print::cstr!(
            "Run `<cyan,bold>cargo help rustdoc</>` for more detailed information.\n"
        ))
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let ws = args.workspace(config)?;
    let output_format = if let Some(output_format) = args._value_of("output-format") {
        config
            .cli_unstable()
            .fail_if_stable_opt("--output-format", 12103)?;
        output_format.parse()?
    } else {
        OutputFormat::Html
    };

    let mut compile_opts = args.compile_options_for_single_package(
        config,
        CompileMode::Doc {
            deps: false,
            json: matches!(output_format, OutputFormat::Json),
        },
        Some(&ws),
        ProfileChecking::Custom,
    )?;
    let target_args = values(args, "args");

    compile_opts.target_rustdoc_args = if target_args.is_empty() {
        None
    } else {
        Some(target_args)
    };

    let doc_opts = DocOptions {
        open_result: args.flag("open"),
        output_format,
        compile_opts,
    };
    ops::doc(&ws, &doc_opts)?;
    Ok(())
}
