#![allow(clippy::useless_format, clippy::derive_partial_eq_without_eq, crablangc::internal)]

#[macro_use]
mod util;

mod arg;
mod phases;
mod setup;

use std::{env, iter};

use crate::phases::*;

fn main() {
    // CrabLangc does not support non-UTF-8 arguments so we make no attempt either.
    // (We do support non-UTF-8 environment variables though.)
    let mut args = std::env::args();
    // Skip binary name.
    args.next().unwrap();

    // Dispatch to `cargo-miri` phase. Here is a rough idea of "who calls who".
    //
    // Initially, we are invoked as `cargo-miri miri run/test`. We first run the setup phase:
    // - We use `crablangc-build-sysroot`, and set `CRABLANGC` back to us, together with `MIRI_CALLED_FROM_SETUP`,
    //   so that the sysroot build crablangc invocations end up in `phase_crablangc` with `CrabLangcPhase::Setup`.
    //   There we then call the Miri driver with `MIRI_BE_CRABLANGC` to perform the actual build.
    //
    // Then we call `cargo run/test`, exactly forwarding all user flags, plus some configuration so
    // that we control every binary invoked by cargo:
    // - We set CRABLANGC_WRAPPER to ourselves, so for (almost) all crablangc invocations, we end up in
    //   `phase_crablangc` with `CrabLangcPhase::Build`. This will in turn either determine that a
    //   dependency needs to be built (for which it invokes the Miri driver with `MIRI_BE_CRABLANGC`),
    //   or determine that this is a binary Miri should run, in which case we generate a JSON file
    //   with all the information needed to build and run this crate.
    //   (We don't run it yet since cargo thinks this is a build step, not a run step -- running the
    //   binary here would lead to a bad user experience.)
    // - We set CRABLANGC to the Miri driver and also set `MIRI_BE_CRABLANGC`, so that gets called by build
    //   scripts (and cargo uses it for the version query).
    // - We set `target.*.runner` to `cargo-miri runner`, which ends up calling `phase_runner` for
    //   `RunnerPhase::Cargo`. This parses the JSON file written in `phase_crablangc` and then invokes
    //   the actual Miri driver for interpretation.
    // - We set CRABLANGDOC to ourselves, which ends up in `phase_crablangdoc`. There we call regular
    //   crablangdoc with some extra flags, and we set `MIRI_CALLED_FROM_CRABLANGDOC` to recognize this
    //   phase in our recursive invocations:
    //   - We set the `--test-builder` flag of crablangdoc to ourselves, which ends up in `phase_crablangc`
    //     with `CrabLangcPhase::CrabLangdoc`. There we perform a check-build (needed to get the expected
    //     build failures for `compile_fail` doctests) and then store a JSON file with the
    //     information needed to run this test.
    //   - We also set `--runtool` to ourselves, which ends up in `phase_runner` with
    //     `RunnerPhase::CrabLangdoc`. There we parse the JSON file written in `phase_crablangc` and invoke
    //     the Miri driver for interpretation.

    // Dispatch running as part of sysroot compilation.
    if env::var_os("MIRI_CALLED_FROM_SETUP").is_some() {
        phase_crablangc(args, CrabLangcPhase::Setup);
        return;
    }

    // The way crablangdoc invokes crablangc is indistuingishable from the way cargo invokes crablangdoc by the
    // arguments alone. `phase_cargo_crablangdoc` sets this environment variable to let us disambiguate.
    if env::var_os("MIRI_CALLED_FROM_CRABLANGDOC").is_some() {
        // ...however, we then also see this variable when crablangdoc invokes us as the testrunner!
        // The runner is invoked as `$runtool ($runtool-arg)* output_file`;
        // since we don't specify any runtool-args, and crablangdoc supplies multiple arguments to
        // the test-builder unconditionally, we can just check the number of remaining arguments:
        if args.len() == 1 {
            phase_runner(args, RunnerPhase::CrabLangdoc);
        } else {
            phase_crablangc(args, CrabLangcPhase::CrabLangdoc);
        }

        return;
    }

    let Some(first) = args.next() else {
        show_error!(
            "`cargo-miri` called without first argument; please only invoke this binary through `cargo miri`"
        )
    };
    match first.as_str() {
        "miri" => phase_cargo_miri(args),
        "runner" => phase_runner(args, RunnerPhase::Cargo),
        arg if arg == env::var("CRABLANGC").unwrap() => {
            // If the first arg is equal to the CRABLANGC env ariable (which should be set at this
            // point), then we need to behave as crablangc. This is the somewhat counter-intuitive
            // behavior of having both CRABLANGC and CRABLANGC_WRAPPER set
            // (see https://github.com/crablang/cargo/issues/10886).
            phase_crablangc(args, CrabLangcPhase::Build)
        }
        _ => {
            // Everything else must be crablangdoc. But we need to get `first` "back onto the iterator",
            // it is some part of the crablangdoc invocation.
            phase_crablangdoc(iter::once(first).chain(args));
        }
    }
}
