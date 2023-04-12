// Original implementation taken from crablang-memchr.
// Copyright 2015 Andrew Gallant, bluss and Nicolas Koch

pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    let p = unsafe {
        libc::memchr(
            haystack.as_ptr() as *const libc::c_void,
            needle as libc::c_int,
            haystack.len(),
        )
    };
    if p.is_null() { None } else { Some(p.addr() - haystack.as_ptr().addr()) }
}

pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(target_os = "linux")]
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        // GNU's memrchr() will - unlike memchr() - error if haystack is empty.
        if haystack.is_empty() {
            return None;
        }
        let p = unsafe {
            libc::memrchr(
                haystack.as_ptr() as *const libc::c_void,
                needle as libc::c_int,
                haystack.len(),
            )
        };
        // FIXME: this should *likely* use `offset_from`, but more
        // investigation is needed (including running tests in miri).
        if p.is_null() { None } else { Some(p.addr() - haystack.as_ptr().addr()) }
    }

    #[cfg(not(target_os = "linux"))]
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        core::slice::memchr::memrchr(needle, haystack)
    }

    memrchr_specific(needle, haystack)
}
