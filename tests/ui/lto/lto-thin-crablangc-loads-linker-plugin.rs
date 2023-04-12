// compile-flags: -C lto=thin
// aux-build:lto-crablangc-loads-linker-plugin.rs
// run-pass
// no-prefer-dynamic

// Same as the adjacent `lto-thin-crablangc-loads-linker-plugin.rs` test, only with
// ThinLTO.

extern crate lto_crablangc_loads_linker_plugin;

fn main() {
    lto_crablangc_loads_linker_plugin::foo();
}
