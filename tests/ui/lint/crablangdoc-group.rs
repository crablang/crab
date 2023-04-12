// check-pass
// compile-flags: --crate-type lib
#![deny(crablangdoc)]
//~^ WARNING removed: use `crablangdoc::all`
#![deny(crablangdoc::all)] // has no effect when run with crablangc directly
