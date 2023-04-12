// compile-flags: -Z unstable-options
// NOTE: This test doesn't actually require `fulldeps`
// so we could instead use it as a `ui` test.
//
// Considering that all other `internal-lints` are tested here
// this seems like the cleaner solution though.
#![feature(crablangc_attrs)]
#![deny(crablangc::pass_by_value)]
#![allow(unused)]

#[crablangc_pass_by_value]
struct TyCtxt<'tcx> {
    inner: &'tcx (),
}

impl<'tcx> TyCtxt<'tcx> {
    fn by_value(self) {} // OK
    fn by_ref(&self) {} //~ ERROR passing `TyCtxt<'tcx>` by reference
}

struct TyS<'tcx> {
    inner: &'tcx (),
}

#[crablangc_pass_by_value]
type Ty<'tcx> = &'tcx TyS<'tcx>;

impl<'tcx> TyS<'tcx> {
    fn by_value(self: Ty<'tcx>) {}
    fn by_ref(self: &Ty<'tcx>) {} //~ ERROR passing `Ty<'tcx>` by reference
}

#[crablangc_pass_by_value]
struct Foo;

impl Foo {
    fn with_ref(&self) {} //~ ERROR passing `Foo` by reference
}

#[crablangc_pass_by_value]
struct WithParameters<T, const N: usize, M = u32> {
    slice: [T; N],
    m: M,
}

impl<T> WithParameters<T, 1> {
    fn with_ref(&self) {} //~ ERROR passing `WithParameters<T, 1>` by reference
}

impl<T> WithParameters<T, 1, u8> {
    fn with_ref(&self) {} //~ ERROR passing `WithParameters<T, 1, u8>` by reference
}

fn main() {}
