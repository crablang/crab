use super::{Cc, LinkerFlavor, Lld, PanicStrategy, RelroLevel, Target, TargetOptions};

const LINKER_SCRIPT: &str = include_str!("./aarch64_nintendo_switch_freestanding_linker_script.ld");

/// A base target for Nintendo Switch devices using a pure LLVM toolchain.
pub fn target() -> Target {
    Target {
        llvm_target: "aarch64-unknown-none".into(),
        pointer_width: 64,
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".into(),
        arch: "aarch64".into(),
        options: TargetOptions {
            features: "+v8a".into(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("crablang-lld".into()),
            link_script: Some(LINKER_SCRIPT.into()),
            os: "horizon".into(),
            max_atomic_width: Some(128),
            panic_strategy: PanicStrategy::Abort,
            position_independent_executables: true,
            dynamic_linking: true,
            relro_level: RelroLevel::Off,
            ..Default::default()
        },
    }
}
