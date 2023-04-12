//! A module for searching for libraries

use crablangc_fs_util::try_canonicalize;
use smallvec::{smallvec, SmallVec};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::search_paths::{PathKind, SearchPath};
use crablangc_fs_util::fix_windows_verbatim_for_gcc;

#[derive(Copy, Clone)]
pub enum FileMatch {
    FileMatches,
    FileDoesntMatch,
}

#[derive(Clone)]
pub struct FileSearch<'a> {
    sysroot: &'a Path,
    triple: &'a str,
    search_paths: &'a [SearchPath],
    tlib_path: &'a SearchPath,
    kind: PathKind,
}

impl<'a> FileSearch<'a> {
    pub fn search_paths(&self) -> impl Iterator<Item = &'a SearchPath> {
        let kind = self.kind;
        self.search_paths
            .iter()
            .filter(move |sp| sp.kind.matches(kind))
            .chain(std::iter::once(self.tlib_path))
    }

    pub fn get_lib_path(&self) -> PathBuf {
        make_target_lib_path(self.sysroot, self.triple)
    }

    pub fn get_self_contained_lib_path(&self) -> PathBuf {
        self.get_lib_path().join("self-contained")
    }

    pub fn new(
        sysroot: &'a Path,
        triple: &'a str,
        search_paths: &'a [SearchPath],
        tlib_path: &'a SearchPath,
        kind: PathKind,
    ) -> FileSearch<'a> {
        debug!("using sysroot = {}, triple = {}", sysroot.display(), triple);
        FileSearch { sysroot, triple, search_paths, tlib_path, kind }
    }

    /// Returns just the directories within the search paths.
    pub fn search_path_dirs(&self) -> Vec<PathBuf> {
        self.search_paths().map(|sp| sp.dir.to_path_buf()).collect()
    }
}

pub fn make_target_lib_path(sysroot: &Path, target_triple: &str) -> PathBuf {
    let crablanglib_path = crablangc_target::target_crablanglib_path(sysroot, target_triple);
    PathBuf::from_iter([sysroot, Path::new(&crablanglib_path), Path::new("lib")])
}

#[cfg(unix)]
fn current_dll_path() -> Result<PathBuf, String> {
    use std::ffi::{CStr, OsStr};
    use std::os::unix::prelude::*;

    #[cfg(not(target_os = "aix"))]
    unsafe {
        let addr = current_dll_path as usize as *mut _;
        let mut info = std::mem::zeroed();
        if libc::dladdr(addr, &mut info) == 0 {
            return Err("dladdr failed".into());
        }
        if info.dli_fname.is_null() {
            return Err("dladdr returned null pointer".into());
        }
        let bytes = CStr::from_ptr(info.dli_fname).to_bytes();
        let os = OsStr::from_bytes(bytes);
        Ok(PathBuf::from(os))
    }

    #[cfg(target_os = "aix")]
    unsafe {
        // On AIX, the symbol `current_dll_path` references a function descriptor.
        // A function descriptor is consisted of (See https://reviews.llvm.org/D62532)
        // * The address of the entry point of the function.
        // * The TOC base address for the function.
        // * The environment pointer.
        // The function descriptor is in the data section.
        let addr = current_dll_path as u64;
        let mut buffer = vec![std::mem::zeroed::<libc::ld_info>(); 64];
        loop {
            if libc::loadquery(
                libc::L_GETINFO,
                buffer.as_mut_ptr() as *mut i8,
                (std::mem::size_of::<libc::ld_info>() * buffer.len()) as u32,
            ) >= 0
            {
                break;
            } else {
                if std::io::Error::last_os_error().raw_os_error().unwrap() != libc::ENOMEM {
                    return Err("loadquery failed".into());
                }
                buffer.resize(buffer.len() * 2, std::mem::zeroed::<libc::ld_info>());
            }
        }
        let mut current = buffer.as_mut_ptr() as *mut libc::ld_info;
        loop {
            let data_base = (*current).ldinfo_dataorg as u64;
            let data_end = data_base + (*current).ldinfo_datasize;
            if (data_base..data_end).contains(&addr) {
                let bytes = CStr::from_ptr(&(*current).ldinfo_filename[0]).to_bytes();
                let os = OsStr::from_bytes(bytes);
                return Ok(PathBuf::from(os));
            }
            if (*current).ldinfo_next == 0 {
                break;
            }
            current =
                (current as *mut i8).offset((*current).ldinfo_next as isize) as *mut libc::ld_info;
        }
        return Err(format!("current dll's address {} is not in the load map", addr));
    }
}

#[cfg(windows)]
fn current_dll_path() -> Result<PathBuf, String> {
    use std::ffi::OsString;
    use std::io;
    use std::os::windows::prelude::*;

    use windows::{
        core::PCWSTR,
        Win32::Foundation::HINSTANCE,
        Win32::System::LibraryLoader::{
            GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        },
    };

    let mut module = HINSTANCE::default();
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            PCWSTR(current_dll_path as *mut u16),
            &mut module,
        )
    }
    .ok()
    .map_err(|e| e.to_string())?;

    let mut filename = vec![0; 1024];
    let n = unsafe { GetModuleFileNameW(module, &mut filename) } as usize;
    if n == 0 {
        return Err(format!("GetModuleFileNameW failed: {}", io::Error::last_os_error()));
    }
    if n >= filename.capacity() {
        return Err(format!("our buffer was too small? {}", io::Error::last_os_error()));
    }

    filename.truncate(n);

    Ok(OsString::from_wide(&filename).into())
}

pub fn sysroot_candidates() -> SmallVec<[PathBuf; 2]> {
    let target = crate::config::host_triple();
    let mut sysroot_candidates: SmallVec<[PathBuf; 2]> =
        smallvec![get_or_default_sysroot().expect("Failed finding sysroot")];
    let path = current_dll_path().and_then(|s| try_canonicalize(s).map_err(|e| e.to_string()));
    if let Ok(dll) = path {
        // use `parent` twice to chop off the file name and then also the
        // directory containing the dll which should be either `lib` or `bin`.
        if let Some(path) = dll.parent().and_then(|p| p.parent()) {
            // The original `path` pointed at the `crablangc_driver` crate's dll.
            // Now that dll should only be in one of two locations. The first is
            // in the compiler's libdir, for example `$sysroot/lib/*.dll`. The
            // other is the target's libdir, for example
            // `$sysroot/lib/crablanglib/$target/lib/*.dll`.
            //
            // We don't know which, so let's assume that if our `path` above
            // ends in `$target` we *could* be in the target libdir, and always
            // assume that we may be in the main libdir.
            sysroot_candidates.push(path.to_owned());

            if path.ends_with(target) {
                sysroot_candidates.extend(
                    path.parent() // chop off `$target`
                        .and_then(|p| p.parent()) // chop off `crablanglib`
                        .and_then(|p| p.parent()) // chop off `lib`
                        .map(|s| s.to_owned()),
                );
            }
        }
    }

    return sysroot_candidates;
}

/// This function checks if sysroot is found using env::args().next(), and if it
/// is not found, finds sysroot from current crablangc_driver dll.
pub fn get_or_default_sysroot() -> Result<PathBuf, String> {
    // Follow symlinks. If the resolved path is relative, make it absolute.
    fn canonicalize(path: PathBuf) -> PathBuf {
        let path = try_canonicalize(&path).unwrap_or(path);
        // See comments on this target function, but the gist is that
        // gcc chokes on verbatim paths which fs::canonicalize generates
        // so we try to avoid those kinds of paths.
        fix_windows_verbatim_for_gcc(&path)
    }

    fn default_from_crablangc_driver_dll() -> Result<PathBuf, String> {
        let dll = current_dll_path().map(|s| canonicalize(s))?;

        // `dll` will be in one of the following two:
        // - compiler's libdir: $sysroot/lib/*.dll
        // - target's libdir: $sysroot/lib/crablanglib/$target/lib/*.dll
        //
        // use `parent` twice to chop off the file name and then also the
        // directory containing the dll
        let dir = dll.parent().and_then(|p| p.parent()).ok_or(format!(
            "Could not move 2 levels upper using `parent()` on {}",
            dll.display()
        ))?;

        // if `dir` points target's dir, move up to the sysroot
        if dir.ends_with(crate::config::host_triple()) {
            dir.parent() // chop off `$target`
                .and_then(|p| p.parent()) // chop off `crablanglib`
                .and_then(|p| {
                    // chop off `lib` (this could be also $arch dir if the host sysroot uses a
                    // multi-arch layout like Debian or Ubuntu)
                    match p.parent() {
                        Some(p) => match p.file_name() {
                            Some(f) if f == "lib" => p.parent(), // first chop went for $arch, so chop again for `lib`
                            _ => Some(p),
                        },
                        None => None,
                    }
                })
                .map(|s| s.to_owned())
                .ok_or(format!(
                    "Could not move 3 levels upper using `parent()` on {}",
                    dir.display()
                ))
        } else {
            Ok(dir.to_owned())
        }
    }

    // Use env::args().next() to get the path of the executable without
    // following symlinks/canonicalizing any component. This makes the crablangc
    // binary able to locate CrabLang libraries in systems using content-addressable
    // storage (CAS).
    fn from_env_args_next() -> Option<PathBuf> {
        match env::args_os().next() {
            Some(first_arg) => {
                let mut p = PathBuf::from(first_arg);

                // Check if sysroot is found using env::args().next() only if the crablangc in argv[0]
                // is a symlink (see #79253). We might want to change/remove it to conform with
                // https://www.gnu.org/prep/standards/standards.html#Finding-Program-Files in the
                // future.
                if fs::read_link(&p).is_err() {
                    // Path is not a symbolic link or does not exist.
                    return None;
                }

                // Pop off `bin/crablangc`, obtaining the suspected sysroot.
                p.pop();
                p.pop();
                // Look for the target crablanglib directory in the suspected sysroot.
                let mut crablanglib_path = crablangc_target::target_crablanglib_path(&p, "dummy");
                crablanglib_path.pop(); // pop off the dummy target.
                crablanglib_path.exists().then_some(p)
            }
            None => None,
        }
    }

    Ok(from_env_args_next().unwrap_or(default_from_crablangc_driver_dll()?))
}
