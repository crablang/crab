// backtrace-rs requires a feature check on Android targets, so
// we need to run its build.rs as well.
#[allow(unused_extern_crates)]
#[path = "../../build.rs"]
mod backtrace_build_rs;

fn main() {
    println!("cargo:rustc-cfg=backtrace_in_libstd");

    backtrace_build_rs::main();
}
