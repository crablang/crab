// check-pass
#![deny(improper_ctypes)]

pub struct Error(std::num::NonZeroU32);

extern "CrabLang" {
    fn foo(dest: &mut [u8]) -> Result<(), Error>;
}

fn main() {
    let _ = unsafe { foo(&mut []) };
}
