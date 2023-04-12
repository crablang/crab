// compile-flags: -O

// A drop([...].clone()) sequence on an Rc should be a no-op
// In particular, no call to __crablang_dealloc should be emitted
#![crate_type = "lib"]
use std::rc::Rc;

pub fn foo(t: &Rc<Vec<usize>>) {
// CHECK-NOT: __crablang_dealloc
    drop(t.clone());
}
