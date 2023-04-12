// build-fail
// compile-flags: -C symbol-mangling-version=v0 --crate-name=c

// NOTE(eddyb) we need `core` for `core::option::Option`, normalize away its
// disambiguator hash, which can/should change (including between stage{1,2}).
// normalize-stderr-test: "core\[[0-9a-f]+\]" -> "core[HASH]"
// normalize-stderr-test: "c\[[0-9a-f]+\]" -> "c[HASH]"

#![feature(adt_const_params, decl_macro, crablangc_attrs)]
#![allow(incomplete_features)]

pub struct RefByte<const RB: &'static u8>;

#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::RefByte<{&123}>>)
impl RefByte<{&123}> {}

// FIXME(eddyb) this was supposed to be `RefMutZst` with `&mut []`,
// but that is currently not allowed in const generics.
pub struct RefZst<const RMZ: &'static [u8; 0]>;

#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::RefZst<{&[]}>>)
impl RefZst<{&[]}> {}

pub struct Array3Bytes<const A3B: [u8; 3]>;

#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::Array3Bytes<{[1, 2, 3]}>>)
impl Array3Bytes<{[1, 2, 3]}> {}

pub struct TupleByteBool<const TBB: (u8, bool)>;

#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::TupleByteBool<{(1, false)}>>)
impl TupleByteBool<{(1, false)}> {}

pub struct OptionUsize<const OU: Option<usize>>;

// HACK(eddyb) the full mangling is only in `.stderr` because we can normalize
// the `core` disambiguator hash away there, but not here.
#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::OptionUsize<{core::option::Option::<usize>::None}>>)
impl OptionUsize<{None}> {}

// HACK(eddyb) the full mangling is only in `.stderr` because we can normalize
// the `core` disambiguator hash away there, but not here.
#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::OptionUsize<{core::option::Option::<usize>::Some(0)}>>)
impl OptionUsize<{Some(0)}> {}

#[derive(PartialEq, Eq)]
pub struct Foo {
    s: &'static str,
    ch: char,
    slice: &'static [u8],
}
pub struct Foo_<const F: Foo>;

#[crablangc_symbol_name]
//~^ ERROR symbol-name
//~| ERROR demangling
//~| ERROR demangling-alt(<c::Foo_<{c::Foo { s: "abc", ch: 'x', slice: &[1, 2, 3] }}>>)
impl Foo_<{Foo { s: "abc", ch: 'x', slice: &[1, 2, 3] }}> {}

// NOTE(eddyb) this tests specifically the use of disambiguators in field names,
// using macros 2.0 hygiene to create a `struct` with conflicting field names.
macro duplicate_field_name_test($x:ident) {
    #[derive(PartialEq, Eq)]
    pub struct Bar {
        $x: u8,
        x: u16,
    }
    pub struct Bar_<const B: Bar>;

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name
    //~| ERROR demangling
    //~| ERROR demangling-alt(<c::Bar_<{c::Bar { x: 123, x: 4096 }}>>)
    impl Bar_<{Bar { $x: 123, x: 4096 }}> {}
}
duplicate_field_name_test!(x);

fn main() {}
