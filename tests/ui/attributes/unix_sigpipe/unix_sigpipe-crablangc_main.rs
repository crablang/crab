// run-pass
// aux-build:sigpipe-utils.rs

#![feature(unix_sigpipe)]
#![feature(crablangc_attrs)]

#[unix_sigpipe = "sig_dfl"]
#[crablangc_main]
fn crablangc_main() {
    extern crate sigpipe_utils;

    // #[unix_sigpipe = "sig_dfl"] is active, so SIGPIPE handler shall be
    // SIG_DFL. Note that we have a #[crablangc_main], but it should still work.
    sigpipe_utils::assert_sigpipe_handler(sigpipe_utils::SignalHandler::Default);
}
