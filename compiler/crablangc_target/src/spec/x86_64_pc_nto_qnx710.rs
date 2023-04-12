use super::nto_qnx_base;
use crate::spec::{Cc, LinkerFlavor, Lld, Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "x86_64-pc-unknown".into(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .into(),
        arch: "x86_64".into(),
        options: TargetOptions {
            cpu: "x86-64".into(),
            max_atomic_width: Some(64),
            pre_link_args: TargetOptions::link_args(
                LinkerFlavor::Gnu(Cc::Yes, Lld::No),
                &["-Vgcc_ntox86_64_cxx"],
            ),
            env: "nto71".into(),
            ..nto_qnx_base::opts()
        },
    }
}
