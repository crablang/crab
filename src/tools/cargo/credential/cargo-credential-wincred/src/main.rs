//! Cargo registry windows credential process.

#[cfg(windows)]
mod win {
    use cargo_credential::{Credential, Error};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::core::PWSTR;
    use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Foundation::TRUE;
    use windows_sys::Win32::Security::Credentials::CredDeleteW;
    use windows_sys::Win32::Security::Credentials::CredReadW;
    use windows_sys::Win32::Security::Credentials::CredWriteW;
    use windows_sys::Win32::Security::Credentials::CREDENTIALW;
    use windows_sys::Win32::Security::Credentials::CRED_PERSIST_LOCAL_MACHINE;
    use windows_sys::Win32::Security::Credentials::CRED_TYPE_GENERIC;

    pub(crate) struct WindowsCredential;

    /// Converts a string to a nul-terminated wide UTF-16 byte sequence.
    fn wstr(s: &str) -> Vec<u16> {
        let mut wide: Vec<u16> = OsStr::new(s).encode_wide().collect();
        if wide.iter().any(|b| *b == 0) {
            panic!("nul byte in wide string");
        }
        wide.push(0);
        wide
    }

    fn target_name(registry_name: &str) -> Vec<u16> {
        wstr(&format!("cargo-registry:{}", registry_name))
    }

    impl Credential for WindowsCredential {
        fn name(&self) -> &'static str {
            env!("CARGO_PKG_NAME")
        }

        fn get(&self, index_url: &str) -> Result<String, Error> {
            let target_name = target_name(index_url);
            let p_credential: *mut CREDENTIALW = std::ptr::null_mut() as *mut _;
            unsafe {
                if CredReadW(
                    target_name.as_ptr(),
                    CRED_TYPE_GENERIC,
                    0,
                    p_credential as *mut _ as *mut _,
                ) != TRUE
                {
                    return Err(format!(
                        "failed to fetch token: {}",
                        std::io::Error::last_os_error()
                    )
                    .into());
                }
                let bytes = std::slice::from_raw_parts(
                    (*p_credential).CredentialBlob,
                    (*p_credential).CredentialBlobSize as usize,
                );
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| "failed to convert token to UTF8".into())
            }
        }

        fn store(&self, index_url: &str, token: &str, name: Option<&str>) -> Result<(), Error> {
            let token = token.as_bytes();
            let target_name = target_name(index_url);
            let comment = match name {
                Some(name) => wstr(&format!("Cargo registry token for {}", name)),
                None => wstr("Cargo registry token"),
            };
            let mut credential = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target_name.as_ptr() as PWSTR,
                Comment: comment.as_ptr() as PWSTR,
                LastWritten: FILETIME {
                    dwLowDateTime: 0,
                    dwHighDateTime: 0,
                },
                CredentialBlobSize: token.len() as u32,
                CredentialBlob: token.as_ptr() as *mut u8,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: std::ptr::null_mut(),
                TargetAlias: std::ptr::null_mut(),
                UserName: std::ptr::null_mut(),
            };
            let result = unsafe { CredWriteW(&mut credential, 0) };
            if result != TRUE {
                let err = std::io::Error::last_os_error();
                return Err(format!("failed to store token: {}", err).into());
            }
            Ok(())
        }

        fn erase(&self, index_url: &str) -> Result<(), Error> {
            let target_name = target_name(index_url);
            let result = unsafe { CredDeleteW(target_name.as_ptr(), CRED_TYPE_GENERIC, 0) };
            if result != TRUE {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
                    eprintln!("not currently logged in to `{}`", index_url);
                    return Ok(());
                }
                return Err(format!("failed to remove token: {}", err).into());
            }
            Ok(())
        }
    }
}

#[cfg(not(windows))]
use cargo_credential::UnsupportedCredential as WindowsCredential;
#[cfg(windows)]
use win::WindowsCredential;

fn main() {
    cargo_credential::main(WindowsCredential);
}
