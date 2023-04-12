use crate::spec::{CodeModel, Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "riscv32-unknown-linux-musl".into(),
        pointer_width: 32,
        data_layout: "e-m:e-p:32:32-i64:64-n32-S128".into(),
        arch: "riscv32".into(),
        options: TargetOptions {
            code_model: Some(CodeModel::Medium),
            cpu: "generic-rv32".into(),
            features: "+m,+a,+f,+d,+c".into(),
            llvm_abiname: "ilp32d".into(),
            max_atomic_width: Some(32),
            ..super::linux_musl_base::opts()
        },
    }
}
