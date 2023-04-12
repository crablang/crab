// Test internal const fn feature gate.

#[crablangc_const_unstable(feature="fzzzzzt")] //~ stability attributes may not be used outside
pub const fn bazinga() {}

fn main() {
}
