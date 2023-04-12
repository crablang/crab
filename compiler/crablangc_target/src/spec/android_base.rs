use crate::spec::{SanitizerSet, TargetOptions};

pub fn opts() -> TargetOptions {
    let mut base = super::linux_base::opts();
    base.os = "android".into();
    base.is_like_android = true;
    base.default_dwarf_version = 2;
    base.has_thread_local = false;
    base.supported_sanitizers = SanitizerSet::ADDRESS;
    // This is for backward compatibility, see https://github.com/crablang/crablang/issues/49867
    // for context. (At that time, there was no `-C force-unwind-tables`, so the only solution
    // was to always emit `uwtable`).
    base.default_uwtable = true;
    base.crt_static_respected = true;
    base
}
