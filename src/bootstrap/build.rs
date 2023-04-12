use std::env;

fn main() {
    let host = env::var("HOST").unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:crablangc-env=BUILD_TRIPLE={}", host);
}
