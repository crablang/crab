// compile-flags: -O
#![crate_type = "lib"]

pub fn foo(t: &mut Vec<usize>) {
    // CHECK-NOT: __crablang_dealloc
    let mut taken = std::mem::take(t);
    taken.pop();
    *t = taken;
}
