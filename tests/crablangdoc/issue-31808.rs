// Test that associated item impls on primitive types don't crash crablangdoc

pub trait Foo {
    const BAR: usize;
    type BAZ;
}

impl Foo for () {
    const BAR: usize = 0;
    type BAZ = usize;
}
