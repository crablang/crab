fn main() {
    // Forward the profile to the main compilation
    println!("cargo:crablangc-env=PROFILE={}", std::env::var("PROFILE").unwrap());
    // Don't rebuild even if nothing changed
    println!("cargo:rerun-if-changed=build.rs");
    crablangc_tools_util::setup_version_info!();
}
