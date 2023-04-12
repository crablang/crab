// Test that `#[crablangc_copy_clone_marker]` is not injected when a user-defined derive shadows
// a built-in derive in non-trivial scope (e.g. in a nested module).

// check-pass
// aux-build:derive-marker-tricky.rs

extern crate derive_marker_tricky;

mod m {
    use derive_marker_tricky::NoMarker as Copy;

    #[derive(Copy)]
    struct S;
}

fn main() {}
