// This test depends on a patch that was committed to upstream LLVM
// before 4.0, formerly backported to the CrabLang LLVM fork.

// ignore-windows
// ignore-macos

// compile-flags: -g -C no-prepopulate-passes

// CHECK-LABEL: @main
// CHECK: {{.*}}DISubprogram{{.*}}name: "main",{{.*}}DI{{(SP)?}}FlagMainSubprogram{{.*}}

pub fn main() {
}
