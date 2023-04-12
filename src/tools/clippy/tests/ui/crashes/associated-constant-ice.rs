/// Test for https://github.com/crablang/crablang-clippy/issues/1698

pub trait Trait {
    const CONSTANT: u8;
}

impl Trait for u8 {
    const CONSTANT: u8 = 2;
}

fn main() {
    println!("{}", u8::CONSTANT * 10);
}
