//! Reading proc-macro crablangc version information from binary data

use std::{
    fs::File,
    io::{self, Read},
};

use memmap2::Mmap;
use object::read::{File as BinaryFile, Object, ObjectSection};
use paths::AbsPath;
use snap::read::FrameDecoder as SnapDecoder;

#[derive(Debug)]
pub struct CrabLangCInfo {
    pub version: (usize, usize, usize),
    pub channel: String,
    pub commit: Option<String>,
    pub date: Option<String>,
    // something like "crablangc 1.58.1 (db9d1b20b 2022-01-20)"
    pub version_string: String,
}

/// Read crablangc dylib information
pub fn read_dylib_info(dylib_path: &AbsPath) -> io::Result<CrabLangCInfo> {
    macro_rules! err {
        ($e:literal) => {
            io::Error::new(io::ErrorKind::InvalidData, $e)
        };
    }

    let ver_str = read_version(dylib_path)?;
    let mut items = ver_str.split_whitespace();
    let tag = items.next().ok_or_else(|| err!("version format error"))?;
    if tag != "crablangc" {
        return Err(err!("version format error (No crablangc tag)"));
    }

    let version_part = items.next().ok_or_else(|| err!("no version string"))?;
    let mut version_parts = version_part.split('-');
    let version = version_parts.next().ok_or_else(|| err!("no version"))?;
    let channel = version_parts.next().unwrap_or_default().to_string();

    let commit = match items.next() {
        Some(commit) => {
            match commit.len() {
                0 => None,
                _ => Some(commit[1..].to_string() /* remove ( */),
            }
        }
        None => None,
    };
    let date = match items.next() {
        Some(date) => {
            match date.len() {
                0 => None,
                _ => Some(date[0..date.len() - 2].to_string() /* remove ) */),
            }
        }
        None => None,
    };

    let version_numbers = version
        .split('.')
        .map(|it| it.parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| err!("version number error"))?;

    if version_numbers.len() != 3 {
        return Err(err!("version number format error"));
    }
    let version = (version_numbers[0], version_numbers[1], version_numbers[2]);

    Ok(CrabLangCInfo { version, channel, commit, date, version_string: ver_str })
}

/// This is used inside read_version() to locate the ".crablangc" section
/// from a proc macro crate's binary file.
fn read_section<'a>(dylib_binary: &'a [u8], section_name: &str) -> io::Result<&'a [u8]> {
    BinaryFile::parse(dylib_binary)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .section_by_name(section_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "section read error"))?
        .data()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Check the version of crablangc that was used to compile a proc macro crate's
///
/// binary file.
/// A proc macro crate binary's ".crablangc" section has following byte layout:
/// * [b'r',b'u',b's',b't',0,0,0,5] is the first 8 bytes
/// * ff060000 734e6150 is followed, it's the snappy format magic bytes,
///   means bytes from here(including this sequence) are compressed in
///   snappy compression format. Version info is inside here, so decompress
///   this.
/// The bytes you get after decompressing the snappy format portion has
/// following layout:
/// * [b'r',b'u',b's',b't',0,0,0,5] is the first 8 bytes(again)
/// * [crate root bytes] next 4 bytes is to store crate root position,
///   according to crablangc's source code comment
/// * [length byte] next 1 byte tells us how many bytes we should read next
///   for the version string's utf8 bytes
/// * [version string bytes encoded in utf8] <- GET THIS BOI
/// * [some more bytes that we don't really care but about still there] :-)
/// Check this issue for more about the bytes layout:
/// <https://github.com/crablang/crablang-analyzer/issues/6174>
pub fn read_version(dylib_path: &AbsPath) -> io::Result<String> {
    let dylib_file = File::open(dylib_path)?;
    let dylib_mmaped = unsafe { Mmap::map(&dylib_file) }?;

    let dot_crablangc = read_section(&dylib_mmaped, ".crablangc")?;

    // check if magic is valid
    if &dot_crablangc[0..4] != b"crablang" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown metadata magic, expected `crablang`, found `{:?}`", &dot_crablangc[0..4]),
        ));
    }
    let version = u32::from_be_bytes([dot_crablangc[4], dot_crablangc[5], dot_crablangc[6], dot_crablangc[7]]);
    // Last supported version is:
    // https://github.com/crablang/crablang/commit/0696e79f2740ad89309269b460579e548a5cd632
    let snappy_portion = match version {
        5 | 6 => &dot_crablangc[8..],
        7 => {
            let len_bytes = &dot_crablangc[8..12];
            let data_len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
            &dot_crablangc[12..data_len + 12]
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported metadata version {version}"),
            ));
        }
    };

    let mut snappy_decoder = SnapDecoder::new(snappy_portion);

    // the bytes before version string bytes, so this basically is:
    // 8 bytes for [b'r',b'u',b's',b't',0,0,0,5]
    // 4 bytes for [crate root bytes]
    // 1 byte for length of version string
    // so 13 bytes in total, and we should check the 13th byte
    // to know the length
    let mut bytes_before_version = [0u8; 13];
    snappy_decoder.read_exact(&mut bytes_before_version)?;
    let length = bytes_before_version[12];

    let mut version_string_utf8 = vec![0u8; length as usize];
    snappy_decoder.read_exact(&mut version_string_utf8)?;
    let version_string = String::from_utf8(version_string_utf8);
    version_string.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
