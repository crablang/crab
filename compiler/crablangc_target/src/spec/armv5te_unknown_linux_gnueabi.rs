use crate::spec::{Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "armv5te-unknown-linux-gnueabi".into(),
        pointer_width: 32,
        data_layout: "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".into(),
        arch: "arm".into(),
        options: TargetOptions {
            abi: "eabi".into(),
            features: "+soft-float,+strict-align".into(),
            // Atomic operations provided by compiler-builtins
            max_atomic_width: Some(32),
            mcount: "\u{1}__gnu_mcount_nc".into(),
            has_thumb_interworking: true,
            ..super::linux_gnu_base::opts()
        },
    }
}
