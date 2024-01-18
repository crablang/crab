fn main() {
    println!("cargo:rerun-if-changed=src/callback.c");

    cc::Build::new()
        .opt_level(0)
        .debug(false)
        .flag("-g1")
        .file("src/callback.c")
        .compile("libcallback.a");
}
