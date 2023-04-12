use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = env::var("TARGET").expect("TARGET was not set");
    if target.contains("freebsd") {
        if env::var("CRABLANG_STD_FREEBSD_12_ABI").is_ok() {
            println!("cargo:crablangc-cfg=freebsd12");
        } else if env::var("CRABLANG_STD_FREEBSD_13_ABI").is_ok() {
            println!("cargo:crablangc-cfg=freebsd12");
            println!("cargo:crablangc-cfg=freebsd13");
        }
    } else if target.contains("linux")
        || target.contains("netbsd")
        || target.contains("dragonfly")
        || target.contains("openbsd")
        || target.contains("solaris")
        || target.contains("illumos")
        || target.contains("apple-darwin")
        || target.contains("apple-ios")
        || target.contains("apple-watchos")
        || target.contains("uwp")
        || target.contains("windows")
        || target.contains("fuchsia")
        || (target.contains("sgx") && target.contains("fortanix"))
        || target.contains("hermit")
        || target.contains("l4re")
        || target.contains("redox")
        || target.contains("haiku")
        || target.contains("vxworks")
        || target.contains("wasm32")
        || target.contains("wasm64")
        || target.contains("asmjs")
        || target.contains("espidf")
        || target.contains("solid")
        || target.contains("nintendo-3ds")
        || target.contains("nto")
    {
        // These platforms don't have any special requirements.
    } else {
        // This is for Cargo's build-std support, to mark std as unstable for
        // typically no_std platforms.
        // This covers:
        // - os=none ("bare metal" targets)
        // - mipsel-sony-psp
        // - nvptx64-nvidia-cuda
        // - arch=avr
        // - tvos (aarch64-apple-tvos, x86_64-apple-tvos)
        // - uefi (x86_64-unknown-uefi, i686-unknown-uefi)
        // - JSON targets
        // - Any new targets that have not been explicitly added above.
        println!("cargo:crablangc-cfg=feature=\"restricted-std\"");
    }
    println!("cargo:crablangc-env=STD_ENV_ARCH={}", env::var("CARGO_CFG_TARGET_ARCH").unwrap());
    println!("cargo:crablangc-cfg=backtrace_in_libstd");
}
