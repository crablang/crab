// ignore-windows

// compile-flags: -g  -C no-prepopulate-passes -Z simulate-remapped-crablang-src-base=/crablangc/xyz

// Here we check that importing std will not cause real path to std source files
// to leak. If crablangc was compiled with remap-debuginfo = true, this should be
// true automatically. If paths to std library hasn't been remapped, we use the
// above simulate-remapped-crablang-src-base option to do it temporarily

// CHECK: !DIFile(filename: "{{/crablangc/.*/library/std/src/panic.rs}}"
fn main() {
    std::thread::spawn(|| {
        println!("hello");
    });
}
