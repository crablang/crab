// Regression test for https://github.com/crablang/crablang/issues/56445#issuecomment-524494170
pub struct Memory<'rom> {
    rom: &'rom [u8],
    ram: [u8; Self::SIZE],
    //~^ ERROR: generic `Self` types are currently not permitted in anonymous constants
}

impl<'rom> Memory<'rom> {
    pub const SIZE: usize = 0x8000;
}

fn main() {}
