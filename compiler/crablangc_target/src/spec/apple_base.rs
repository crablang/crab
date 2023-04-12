use std::{borrow::Cow, env};

use crate::spec::{cvs, Cc, DebuginfoKind, FramePointer, LinkArgs};
use crate::spec::{LinkerFlavor, Lld, SplitDebuginfo, StaticCow, TargetOptions};

#[cfg(test)]
#[path = "apple/tests.rs"]
mod tests;

use Arch::*;
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Arch {
    Armv7,
    Armv7k,
    Armv7s,
    Arm64,
    Arm64_32,
    I386,
    I686,
    X86_64,
    X86_64_sim,
    X86_64_macabi,
    Arm64_macabi,
    Arm64_sim,
}

impl Arch {
    pub fn target_name(self) -> &'static str {
        match self {
            Armv7 => "armv7",
            Armv7k => "armv7k",
            Armv7s => "armv7s",
            Arm64 | Arm64_macabi | Arm64_sim => "arm64",
            Arm64_32 => "arm64_32",
            I386 => "i386",
            I686 => "i686",
            X86_64 | X86_64_sim | X86_64_macabi => "x86_64",
        }
    }

    pub fn target_arch(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Armv7 | Armv7k | Armv7s => "arm",
            Arm64 | Arm64_32 | Arm64_macabi | Arm64_sim => "aarch64",
            I386 | I686 => "x86",
            X86_64 | X86_64_sim | X86_64_macabi => "x86_64",
        })
    }

    fn target_abi(self) -> &'static str {
        match self {
            Armv7 | Armv7k | Armv7s | Arm64 | Arm64_32 | I386 | I686 | X86_64 => "",
            X86_64_macabi | Arm64_macabi => "macabi",
            // x86_64-apple-ios is a simulator target, even though it isn't
            // declared that way in the target like the other ones...
            Arm64_sim | X86_64_sim => "sim",
        }
    }

    fn target_cpu(self) -> &'static str {
        match self {
            Armv7 => "cortex-a8", // iOS7 is supported on iPhone 4 and higher
            Armv7k => "cortex-a8",
            Armv7s => "cortex-a9",
            Arm64 => "apple-a7",
            Arm64_32 => "apple-s4",
            I386 | I686 => "yonah",
            X86_64 | X86_64_sim => "core2",
            X86_64_macabi => "core2",
            Arm64_macabi => "apple-a12",
            Arm64_sim => "apple-a12",
        }
    }
}

fn pre_link_args(os: &'static str, arch: Arch, abi: &'static str) -> LinkArgs {
    let platform_name: StaticCow<str> = match abi {
        "sim" => format!("{os}-simulator").into(),
        "macabi" => "mac-catalyst".into(),
        _ => os.into(),
    };

    let platform_version: StaticCow<str> = match os {
        "ios" => ios_lld_platform_version(),
        "tvos" => tvos_lld_platform_version(),
        "watchos" => watchos_lld_platform_version(),
        "macos" => macos_lld_platform_version(arch),
        _ => unreachable!(),
    }
    .into();

    let arch = arch.target_name();

    let mut args = TargetOptions::link_args(
        LinkerFlavor::Darwin(Cc::No, Lld::No),
        &["-arch", arch, "-platform_version"],
    );
    super::add_link_args_iter(
        &mut args,
        LinkerFlavor::Darwin(Cc::No, Lld::No),
        [platform_name, platform_version.clone(), platform_version].into_iter(),
    );
    if abi != "macabi" {
        super::add_link_args(&mut args, LinkerFlavor::Darwin(Cc::Yes, Lld::No), &["-arch", arch]);
    }

    args
}

pub fn opts(os: &'static str, arch: Arch) -> TargetOptions {
    // Static TLS is only available in macOS 10.7+. If you try to compile for 10.6
    // either the linker will complain if it is used or the binary will end up
    // segfaulting at runtime when run on 10.6. CrabLang by default supports macOS
    // 10.7+, but there is a standard environment variable,
    // MACOSX_DEPLOYMENT_TARGET, which is used to signal targeting older
    // versions of macOS. For example compiling on 10.10 with
    // MACOSX_DEPLOYMENT_TARGET set to 10.6 will cause the linker to generate
    // warnings about the usage of static TLS.
    //
    // Here we detect what version is being requested, defaulting to 10.7. Static
    // TLS is flagged as enabled if it looks to be supported. The architecture
    // only matters for default deployment target which is 11.0 for ARM64 and
    // 10.7 for everything else.
    let has_thread_local = os == "macos" && macos_deployment_target(Arch::X86_64) >= (10, 7);

    let abi = arch.target_abi();

    TargetOptions {
        abi: abi.into(),
        os: os.into(),
        cpu: arch.target_cpu().into(),
        link_env_remove: link_env_remove(arch, os),
        vendor: "apple".into(),
        linker_flavor: LinkerFlavor::Darwin(Cc::Yes, Lld::No),
        // macOS has -dead_strip, which doesn't rely on function_sections
        function_sections: false,
        dynamic_linking: true,
        pre_link_args: pre_link_args(os, arch, abi),
        families: cvs!["unix"],
        is_like_osx: true,
        default_dwarf_version: 2,
        frame_pointer: FramePointer::Always,
        has_rpath: true,
        dll_suffix: ".dylib".into(),
        archive_format: "darwin".into(),
        has_thread_local,
        abi_return_struct_as_int: true,
        emit_debug_gdb_scripts: false,
        eh_frame_header: false,

        debuginfo_kind: DebuginfoKind::DwarfDsym,
        // The historical default for macOS targets is to run `dsymutil` which
        // generates a packed version of debuginfo split from the main file.
        split_debuginfo: SplitDebuginfo::Packed,
        supported_split_debuginfo: Cow::Borrowed(&[
            SplitDebuginfo::Packed,
            SplitDebuginfo::Unpacked,
            SplitDebuginfo::Off,
        ]),

        // This environment variable is pretty magical but is intended for
        // producing deterministic builds. This was first discovered to be used
        // by the `ar` tool as a way to control whether or not mtime entries in
        // the archive headers were set to zero or not. It appears that
        // eventually the linker got updated to do the same thing and now reads
        // this environment variable too in recent versions.
        //
        // For some more info see the commentary on #47086
        link_env: Cow::Borrowed(&[(Cow::Borrowed("ZERO_AR_DATE"), Cow::Borrowed("1"))]),

        ..Default::default()
    }
}

fn deployment_target(var_name: &str) -> Option<(u32, u32)> {
    let deployment_target = env::var(var_name).ok();
    deployment_target
        .as_ref()
        .and_then(|s| s.split_once('.'))
        .and_then(|(a, b)| a.parse::<u32>().and_then(|a| b.parse::<u32>().map(|b| (a, b))).ok())
}

fn macos_default_deployment_target(arch: Arch) -> (u32, u32) {
    // Note: Arm64_sim is not included since macOS has no simulator.
    if matches!(arch, Arm64 | Arm64_macabi) { (11, 0) } else { (10, 7) }
}

fn macos_deployment_target(arch: Arch) -> (u32, u32) {
    deployment_target("MACOSX_DEPLOYMENT_TARGET")
        .unwrap_or_else(|| macos_default_deployment_target(arch))
}

fn macos_lld_platform_version(arch: Arch) -> String {
    let (major, minor) = macos_deployment_target(arch);
    format!("{major}.{minor}")
}

pub fn macos_llvm_target(arch: Arch) -> String {
    let (major, minor) = macos_deployment_target(arch);
    format!("{}-apple-macosx{}.{}.0", arch.target_name(), major, minor)
}

fn link_env_remove(arch: Arch, os: &'static str) -> StaticCow<[StaticCow<str>]> {
    // Apple platforms only officially support macOS as a host for any compilation.
    //
    // If building for macOS, we go ahead and remove any erroneous environment state
    // that's only applicable to cross-OS compilation. Always leave anything for the
    // host OS alone though.
    if os == "macos" {
        let mut env_remove = Vec::with_capacity(2);
        // Remove the `SDKROOT` environment variable if it's clearly set for the wrong platform, which
        // may occur when we're linking a custom build script while targeting iOS for example.
        if let Ok(sdkroot) = env::var("SDKROOT") {
            if sdkroot.contains("iPhoneOS.platform") || sdkroot.contains("iPhoneSimulator.platform")
            {
                env_remove.push("SDKROOT".into())
            }
        }
        // Additionally, `IPHONEOS_DEPLOYMENT_TARGET` must not be set when using the Xcode linker at
        // "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld",
        // although this is apparently ignored when using the linker at "/usr/bin/ld".
        env_remove.push("IPHONEOS_DEPLOYMENT_TARGET".into());
        env_remove.into()
    } else {
        // Otherwise if cross-compiling for a different OS/SDK, remove any part
        // of the linking environment that's wrong and reversed.
        match arch {
            Armv7 | Armv7k | Armv7s | Arm64 | Arm64_32 | I386 | I686 | X86_64 | X86_64_sim
            | Arm64_sim => {
                cvs!["MACOSX_DEPLOYMENT_TARGET"]
            }
            X86_64_macabi | Arm64_macabi => cvs!["IPHONEOS_DEPLOYMENT_TARGET"],
        }
    }
}

fn ios_deployment_target() -> (u32, u32) {
    deployment_target("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or((7, 0))
}

pub fn ios_llvm_target(arch: Arch) -> String {
    // Modern iOS tooling extracts information about deployment target
    // from LC_BUILD_VERSION. This load command will only be emitted when
    // we build with a version specific `llvm_target`, with the version
    // set high enough. Luckily one LC_BUILD_VERSION is enough, for Xcode
    // to pick it up (since std and core are still built with the fallback
    // of version 7.0 and hence emit the old LC_IPHONE_MIN_VERSION).
    let (major, minor) = ios_deployment_target();
    format!("{}-apple-ios{}.{}.0", arch.target_name(), major, minor)
}

fn ios_lld_platform_version() -> String {
    let (major, minor) = ios_deployment_target();
    format!("{major}.{minor}")
}

pub fn ios_sim_llvm_target(arch: Arch) -> String {
    let (major, minor) = ios_deployment_target();
    format!("{}-apple-ios{}.{}.0-simulator", arch.target_name(), major, minor)
}

fn tvos_deployment_target() -> (u32, u32) {
    deployment_target("TVOS_DEPLOYMENT_TARGET").unwrap_or((7, 0))
}

fn tvos_lld_platform_version() -> String {
    let (major, minor) = tvos_deployment_target();
    format!("{major}.{minor}")
}

fn watchos_deployment_target() -> (u32, u32) {
    deployment_target("WATCHOS_DEPLOYMENT_TARGET").unwrap_or((5, 0))
}

fn watchos_lld_platform_version() -> String {
    let (major, minor) = watchos_deployment_target();
    format!("{major}.{minor}")
}

pub fn watchos_sim_llvm_target(arch: Arch) -> String {
    let (major, minor) = watchos_deployment_target();
    format!("{}-apple-watchos{}.{}.0-simulator", arch.target_name(), major, minor)
}
