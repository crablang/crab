//@ignore-target-windows: No libc on Windows
//@compile-flags: -Zmiri-disable-isolation

// FIXME: standard handles cannot be closed (https://github.com/crablang/crablang/issues/40032)

fn main() {
    unsafe {
        libc::close(1); //~ ERROR: cannot close stdout
    }
}
