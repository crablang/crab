//! Parses ELF auxiliary vectors.
#![allow(dead_code)]

pub(crate) const AT_NULL: usize = 0;

/// Key to access the CPU Hardware capabilities bitfield.
pub(crate) const AT_HWCAP: usize = 16;
/// Key to access the CPU Hardware capabilities 2 bitfield.
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "powerpc",
    target_arch = "powerpc64"
))]
pub(crate) const AT_HWCAP2: usize = 26;

/// Cache HWCAP bitfields of the ELF Auxiliary Vector.
///
/// If an entry cannot be read all the bits in the bitfield are set to zero.
/// This should be interpreted as all the features being disabled.
#[derive(Debug, Copy, Clone)]
pub(crate) struct AuxVec {
    pub hwcap: usize,
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64"
    ))]
    pub hwcap2: usize,
}

/// ELF Auxiliary Vector
///
/// The auxiliary vector is a memory region in a running ELF program's stack
/// composed of (key: usize, value: usize) pairs.
///
/// The keys used in the aux vector are platform dependent. For Linux, they are
/// defined in [linux/auxvec.h][auxvec_h]. The hardware capabilities of a given
/// CPU can be queried with the  `AT_HWCAP` and `AT_HWCAP2` keys.
///
/// There is no perfect way of reading the auxiliary vector.
///
/// - If the `std_detect_dlsym_getauxval` cargo feature is enabled, this will use
/// `getauxval` if its linked to the binary, and otherwise proceed to a fallback implementation.
/// When `std_detect_dlsym_getauxval` is disabled, this will assume that `getauxval` is
/// linked to the binary - if that is not the case the behavior is undefined.
/// - Otherwise, if the `std_detect_file_io` cargo feature is enabled, it will
///   try to read `/proc/self/auxv`.
/// - If that fails, this function returns an error.
///
/// Note that run-time feature detection is not invoked for features that can
/// be detected at compile-time. Also note that if this function returns an
/// error, cpuinfo still can (and will) be used to try to perform run-time
/// feature detection on some platforms.
///
///  Note: The `std_detect_dlsym_getauxval` cargo feature is ignored on
/// `*-linux-gnu*` and `*-android*` targets because we can safely assume `getauxval`
/// is linked to the binary.
/// - `*-linux-gnu*` targets ([since Rust 1.64](https://blog.rust-lang.org/2022/08/01/Increasing-glibc-kernel-requirements.html))
///   have glibc requirements higher than [glibc 2.16 that added `getauxval`](https://sourceware.org/legacy-ml/libc-announce/2012/msg00000.html).
/// - `*-android*` targets ([since Rust 1.68](https://blog.rust-lang.org/2023/01/09/android-ndk-update-r25.html))
///   have the minimum supported API level higher than [Android 4.3 (API level 18) that added `getauxval`](https://github.com/aosp-mirror/platform_bionic/blob/d3ebc2f7c49a9893b114124d4a6b315f3a328764/libc/include/sys/auxv.h#L49).
///
/// For more information about when `getauxval` is available check the great
/// [`auxv` crate documentation][auxv_docs].
///
/// [auxvec_h]: https://github.com/torvalds/linux/blob/master/include/uapi/linux/auxvec.h
/// [auxv_docs]: https://docs.rs/auxv/0.3.3/auxv/
pub(crate) fn auxv() -> Result<AuxVec, ()> {
    #[cfg(all(
        feature = "std_detect_dlsym_getauxval",
        not(all(target_os = "linux", target_env = "gnu")),
        // TODO: libc crate currently doesn't provide getauxval on 32-bit Android.
        not(all(target_os = "android", target_pointer_width = "64")),
    ))]
    {
        // Try to call a dynamically-linked getauxval function.
        if let Ok(hwcap) = getauxval(AT_HWCAP) {
            // Targets with only AT_HWCAP:
            #[cfg(any(
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "mips",
                target_arch = "mips64"
            ))]
            {
                // Zero could indicate that no features were detected, but it's also used to
                // indicate an error. In either case, try the fallback.
                if hwcap != 0 {
                    return Ok(AuxVec { hwcap });
                }
            }

            // Targets with AT_HWCAP and AT_HWCAP2:
            #[cfg(any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc",
                target_arch = "powerpc64"
            ))]
            {
                if let Ok(hwcap2) = getauxval(AT_HWCAP2) {
                    // Zero could indicate that no features were detected, but it's also used to
                    // indicate an error. In particular, on many platforms AT_HWCAP2 will be
                    // legitimately zero, since it contains the most recent feature flags. Use the
                    // fallback only if no features were detected at all.
                    if hwcap != 0 || hwcap2 != 0 {
                        return Ok(AuxVec { hwcap, hwcap2 });
                    }
                }
            }

            // Intentionnaly not used
            let _ = hwcap;
        }
    }

    #[cfg(any(
        not(feature = "std_detect_dlsym_getauxval"),
        all(target_os = "linux", target_env = "gnu"),
        // TODO: libc crate currently doesn't provide getauxval on 32-bit Android.
        all(target_os = "android", target_pointer_width = "64"),
    ))]
    {
        // Targets with only AT_HWCAP:
        #[cfg(any(
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "mips",
            target_arch = "mips64"
        ))]
        {
            let hwcap = unsafe { libc::getauxval(AT_HWCAP as libc::c_ulong) as usize };
            // Zero could indicate that no features were detected, but it's also used to indicate
            // an error. In either case, try the fallback.
            if hwcap != 0 {
                return Ok(AuxVec { hwcap });
            }
        }

        // Targets with AT_HWCAP and AT_HWCAP2:
        #[cfg(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "powerpc",
            target_arch = "powerpc64"
        ))]
        {
            let hwcap = unsafe { libc::getauxval(AT_HWCAP as libc::c_ulong) as usize };
            let hwcap2 = unsafe { libc::getauxval(AT_HWCAP2 as libc::c_ulong) as usize };
            // Zero could indicate that no features were detected, but it's also used to indicate
            // an error. In particular, on many platforms AT_HWCAP2 will be legitimately zero,
            // since it contains the most recent feature flags. Use the fallback only if no
            // features were detected at all.
            if hwcap != 0 || hwcap2 != 0 {
                return Ok(AuxVec { hwcap, hwcap2 });
            }
        }
    }

    #[cfg(feature = "std_detect_file_io")]
    {
        // If calling getauxval fails, try to read the auxiliary vector from
        // its file:
        auxv_from_file("/proc/self/auxv")
    }
    #[cfg(not(feature = "std_detect_file_io"))]
    {
        Err(())
    }
}

/// Tries to read the `key` from the auxiliary vector by calling the
/// dynamically-linked `getauxval` function. If the function is not linked,
/// this function return `Err`.
#[cfg(all(
    feature = "std_detect_dlsym_getauxval",
    not(all(target_os = "linux", target_env = "gnu"))
))]
fn getauxval(key: usize) -> Result<usize, ()> {
    use libc;
    pub type F = unsafe extern "C" fn(usize) -> usize;
    unsafe {
        let ptr = libc::dlsym(libc::RTLD_DEFAULT, "getauxval\0".as_ptr() as *const _);
        if ptr.is_null() {
            return Err(());
        }

        let ffi_getauxval: F = core::mem::transmute(ptr);
        Ok(ffi_getauxval(key))
    }
}

/// Tries to read the auxiliary vector from the `file`. If this fails, this
/// function returns `Err`.
#[cfg(feature = "std_detect_file_io")]
pub(super) fn auxv_from_file(file: &str) -> Result<AuxVec, ()> {
    let file = super::read_file(file)?;

    // See <https://github.com/torvalds/linux/blob/v5.15/include/uapi/linux/auxvec.h>.
    //
    // The auxiliary vector contains at most 34 (key,value) fields: from
    // `AT_MINSIGSTKSZ` to `AT_NULL`, but its number may increase.
    let len = file.len();
    let mut buf = alloc::vec![0_usize; 1 + len / core::mem::size_of::<usize>()];
    unsafe {
        core::ptr::copy_nonoverlapping(file.as_ptr(), buf.as_mut_ptr() as *mut u8, len);
    }

    auxv_from_buf(&buf)
}

/// Tries to interpret the `buffer` as an auxiliary vector. If that fails, this
/// function returns `Err`.
#[cfg(feature = "std_detect_file_io")]
fn auxv_from_buf(buf: &[usize]) -> Result<AuxVec, ()> {
    // Targets with only AT_HWCAP:
    #[cfg(any(
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "mips",
        target_arch = "mips64",
    ))]
    {
        for el in buf.chunks(2) {
            match el[0] {
                AT_NULL => break,
                AT_HWCAP => return Ok(AuxVec { hwcap: el[1] }),
                _ => (),
            }
        }
    }
    // Targets with AT_HWCAP and AT_HWCAP2:
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64"
    ))]
    {
        let mut hwcap = None;
        // For some platforms, AT_HWCAP2 was added recently, so let it default to zero.
        let mut hwcap2 = 0;
        for el in buf.chunks(2) {
            match el[0] {
                AT_NULL => break,
                AT_HWCAP => hwcap = Some(el[1]),
                AT_HWCAP2 => hwcap2 = el[1],
                _ => (),
            }
        }

        if let Some(hwcap) = hwcap {
            return Ok(AuxVec { hwcap, hwcap2 });
        }
    }
    // Suppress unused variable
    let _ = buf;
    Err(())
}

#[cfg(test)]
mod tests {
    extern crate auxv as auxv_crate;
    use super::*;

    // Reads the Auxiliary Vector key from /proc/self/auxv
    // using the auxv crate.
    #[cfg(feature = "std_detect_file_io")]
    fn auxv_crate_getprocfs(key: usize) -> Option<usize> {
        use self::auxv_crate::procfs::search_procfs_auxv;
        use self::auxv_crate::AuxvType;
        let k = key as AuxvType;
        match search_procfs_auxv(&[k]) {
            Ok(v) => Some(v[&k] as usize),
            Err(_) => None,
        }
    }

    // Reads the Auxiliary Vector key from getauxval()
    // using the auxv crate.
    #[cfg(not(any(target_arch = "mips", target_arch = "mips64")))]
    fn auxv_crate_getauxval(key: usize) -> Option<usize> {
        use self::auxv_crate::getauxval::Getauxval;
        use self::auxv_crate::AuxvType;
        let q = auxv_crate::getauxval::NativeGetauxval {};
        match q.getauxval(key as AuxvType) {
            Ok(v) => Some(v as usize),
            Err(_) => None,
        }
    }

    // FIXME: on mips/mips64 getauxval returns 0, and /proc/self/auxv
    // does not always contain the AT_HWCAP key under qemu.
    #[cfg(any(
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64"
    ))]
    #[test]
    fn auxv_crate() {
        let v = auxv();
        if let Some(hwcap) = auxv_crate_getauxval(AT_HWCAP) {
            let rt_hwcap = v.expect("failed to find hwcap key").hwcap;
            assert_eq!(rt_hwcap, hwcap);
        }

        // Targets with AT_HWCAP and AT_HWCAP2:
        #[cfg(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "powerpc",
            target_arch = "powerpc64"
        ))]
        {
            if let Some(hwcap2) = auxv_crate_getauxval(AT_HWCAP2) {
                let rt_hwcap2 = v.expect("failed to find hwcap2 key").hwcap2;
                assert_eq!(rt_hwcap2, hwcap2);
            }
        }
    }

    #[test]
    fn auxv_dump() {
        if let Ok(auxvec) = auxv() {
            println!("{:?}", auxvec);
        } else {
            println!("both getauxval() and reading /proc/self/auxv failed!");
        }
    }

    #[cfg(feature = "std_detect_file_io")]
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "arm")] {
            #[test]
            fn linux_rpi3() {
                let file = concat!(env!("CARGO_MANIFEST_DIR"), "/src/detect/test_data/linux-rpi3.auxv");
                println!("file: {file}");
                let v = auxv_from_file(file).unwrap();
                assert_eq!(v.hwcap, 4174038);
                assert_eq!(v.hwcap2, 16);
            }

            #[test]
            fn linux_macos_vb() {
                let file = concat!(env!("CARGO_MANIFEST_DIR"), "/src/detect/test_data/macos-virtualbox-linux-x86-4850HQ.auxv");
                println!("file: {file}");
                // The file contains HWCAP but not HWCAP2. In that case, we treat HWCAP2 as zero.
                let v = auxv_from_file(file).unwrap();
                assert_eq!(v.hwcap, 126614527);
                assert_eq!(v.hwcap2, 0);
            }
        } else if #[cfg(target_arch = "aarch64")] {
            #[test]
            fn linux_artificial_aarch64() {
                let file = concat!(env!("CARGO_MANIFEST_DIR"), "/src/detect/test_data/linux-artificial-aarch64.auxv");
                println!("file: {file}");
                let v = auxv_from_file(file).unwrap();
                assert_eq!(v.hwcap, 0x0123456789abcdef);
                assert_eq!(v.hwcap2, 0x02468ace13579bdf);
            }
            #[test]
            fn linux_no_hwcap2_aarch64() {
                let file = concat!(env!("CARGO_MANIFEST_DIR"), "/src/detect/test_data/linux-no-hwcap2-aarch64.auxv");
                println!("file: {file}");
                let v = auxv_from_file(file).unwrap();
                // An absent HWCAP2 is treated as zero, and does not prevent acceptance of HWCAP.
                assert_ne!(v.hwcap, 0);
                assert_eq!(v.hwcap2, 0);
            }
        }
    }

    #[test]
    #[cfg(feature = "std_detect_file_io")]
    fn auxv_dump_procfs() {
        if let Ok(auxvec) = auxv_from_file("/proc/self/auxv") {
            println!("{:?}", auxvec);
        } else {
            println!("reading /proc/self/auxv failed!");
        }
    }

    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64"
    ))]
    #[test]
    #[cfg(feature = "std_detect_file_io")]
    fn auxv_crate_procfs() {
        let v = auxv();
        if let Some(hwcap) = auxv_crate_getprocfs(AT_HWCAP) {
            assert_eq!(v.unwrap().hwcap, hwcap);
        }

        // Targets with AT_HWCAP and AT_HWCAP2:
        #[cfg(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "powerpc",
            target_arch = "powerpc64"
        ))]
        {
            if let Some(hwcap2) = auxv_crate_getprocfs(AT_HWCAP2) {
                assert_eq!(v.unwrap().hwcap2, hwcap2);
            }
        }
    }
}
