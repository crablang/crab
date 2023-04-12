// run-pass
// pretty-expanded FIXME #23616

#[repr(C)]
pub struct Foo(u32);

// ICE trigger, bad handling of differing types between crablang and external ABIs
pub extern "C" fn bar() -> Foo {
    Foo(0)
}

fn main() {}
