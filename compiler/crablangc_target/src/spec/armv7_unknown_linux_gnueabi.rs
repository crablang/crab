use crate::spec::{Target, TargetOptions};

// This target is for glibc Linux on ARMv7 without thumb-mode, NEON or
// hardfloat.

pub fn target() -> Target {
    Target {
        llvm_target: "armv7-unknown-linux-gnueabi".into(),
        pointer_width: 32,
        data_layout: "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".into(),
        arch: "arm".into(),
        options: TargetOptions {
            abi: "eabi".into(),
            features: "+v7,+thumb2,+soft-float,-neon".into(),
            max_atomic_width: Some(64),
            mcount: "\u{1}__gnu_mcount_nc".into(),
            ..super::linux_gnu_base::opts()
        },
    }
}
