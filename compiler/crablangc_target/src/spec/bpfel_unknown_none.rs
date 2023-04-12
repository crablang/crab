use crate::spec::Target;
use crate::{abi::Endian, spec::bpf_base};

pub fn target() -> Target {
    Target {
        llvm_target: "bpfel".into(),
        data_layout: "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128".into(),
        pointer_width: 64,
        arch: "bpf".into(),
        options: bpf_base::opts(Endian::Little),
    }
}
