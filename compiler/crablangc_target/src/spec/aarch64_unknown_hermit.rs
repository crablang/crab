use crate::spec::Target;

pub fn target() -> Target {
    let mut base = super::hermit_base::opts();
    base.max_atomic_width = Some(128);
    base.features = "+v8a,+strict-align,+neon,+fp-armv8".into();

    Target {
        llvm_target: "aarch64-unknown-hermit".into(),
        pointer_width: 64,
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".into(),
        arch: "aarch64".into(),
        options: base,
    }
}
