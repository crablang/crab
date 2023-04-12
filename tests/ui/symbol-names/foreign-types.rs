// build-fail
// compile-flags: -C symbol-mangling-version=v0

#![feature(extern_types)]
#![feature(crablangc_attrs)]

extern "C" {
    type ForeignType;
}

struct Check<T: ?Sized>(T);

#[crablangc_symbol_name]
//~^ ERROR symbol-name(_RMCs
//~| ERROR demangling(<foreign_types[
//~| ERROR demangling-alt(<foreign_types::Check<foreign_types::ForeignType>>)
impl Check<ForeignType> {}

fn main() {}
