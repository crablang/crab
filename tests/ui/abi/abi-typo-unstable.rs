// crablang-intrinsic is unstable and not enabled, so it should not be suggested as a fix
extern "crablang-intrinsec" fn crablang_intrinsic() {} //~ ERROR invalid ABI

fn main() {
    crablang_intrinsic();
}
