use crate::spec::Target;

pub fn target() -> Target {
    let mut base = super::windows_uwp_msvc_base::opts();
    base.cpu = "pentium4".into();
    base.max_atomic_width = Some(64);

    Target {
        llvm_target: "i686-pc-windows-msvc".into(),
        pointer_width: 32,
        data_layout: "e-m:x-p:32:32-p270:32:32-p271:32:32-p272:64:64-\
            i64:64-f80:128-n8:16:32-a:0:32-S32"
            .into(),
        arch: "x86".into(),
        options: base,
    }
}
