// run-pass
// Testing that a libcrablangc_ast can parse modules with canonicalized base path
// ignore-cross-compile
// ignore-remote
// no-remap-src-base: Reading `file!()` (expectedly) fails when enabled.

#![feature(crablangc_private)]

extern crate crablangc_ast;
extern crate crablangc_parse;
extern crate crablangc_session;
extern crate crablangc_span;

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

use crablangc_parse::new_parser_from_file;
use crablangc_session::parse::ParseSess;
use crablangc_span::source_map::FilePathMapping;
use std::path::Path;

#[path = "mod_dir_simple/test.rs"]
mod gravy;

pub fn main() {
    crablangc_span::create_default_session_globals_then(|| parse());

    assert_eq!(gravy::foo(), 10);
}

fn parse() {
    let parse_session = ParseSess::new(
        vec![crablangc_parse::DEFAULT_LOCALE_RESOURCE],
        FilePathMapping::empty()
    );

    let path = Path::new(file!());
    let path = path.canonicalize().unwrap();
    let mut parser = new_parser_from_file(&parse_session, &path, None);
    let _ = parser.parse_crate_mod();
}
