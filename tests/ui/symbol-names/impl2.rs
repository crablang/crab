// build-fail

#![feature(crablangc_attrs)]
#![allow(dead_code)]

trait Foo {
    fn baz();
}

impl Foo for [u8; 1 + 2] {
    #[crablangc_def_path] //~ ERROR def-path(<[u8; 1 + 2] as Foo>::baz)
    fn baz() {}
}

fn main() {}
