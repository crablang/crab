#![feature(type_alias_impl_trait)]

// check-pass

// issue: https://github.com/crablang/crablang/issues/104551

fn main() {
    type T = impl Sized;
    let (_a, _b): T = (1u32, 2u32);
}
