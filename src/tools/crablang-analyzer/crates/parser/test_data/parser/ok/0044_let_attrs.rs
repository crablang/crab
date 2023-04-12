// https://github.com/crablang/crablang-analyzer/issues/677
fn main() {
    #[cfg(feature = "backtrace")]
    let exit_code = panic::catch_unwind(move || main());
}
