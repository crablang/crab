// Targets the Cortex-M23 processor (Baseline ARMv8-M)

use crate::spec::{Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "thumbv8m.base-none-eabi".into(),
        pointer_width: 32,
        data_layout: "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".into(),
        arch: "arm".into(),

        options: TargetOptions {
            abi: "eabi".into(),
            // ARMv8-M baseline doesn't support unaligned loads/stores so we disable them
            // with +strict-align.
            features: "+strict-align".into(),
            max_atomic_width: Some(32),
            ..super::thumb_base::opts()
        },
    }
}
