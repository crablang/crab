// compile-flags: -O
// only-x86_64
// ignore-debug

#![crate_type = "lib"]

// CHECK-LABEL: @vec_zero_bytes
#[no_mangle]
pub fn vec_zero_bytes(n: usize) -> Vec<u8> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(
    // CHECK-NOT: call {{.*}}llvm.memset

    // CHECK: call {{.*}}__crablang_alloc_zeroed(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(
    // CHECK-NOT: call {{.*}}llvm.memset

    // CHECK: ret void
    vec![0; n]
}

// CHECK-LABEL: @vec_one_bytes
#[no_mangle]
pub fn vec_one_bytes(n: usize) -> Vec<u8> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: call {{.*}}__crablang_alloc(
    // CHECK: call {{.*}}llvm.memset

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: ret void
    vec![1; n]
}

// CHECK-LABEL: @vec_zero_scalar
#[no_mangle]
pub fn vec_zero_scalar(n: usize) -> Vec<i32> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: call {{.*}}__crablang_alloc_zeroed(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: ret void
    vec![0; n]
}

// CHECK-LABEL: @vec_one_scalar
#[no_mangle]
pub fn vec_one_scalar(n: usize) -> Vec<i32> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: call {{.*}}__crablang_alloc(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: ret void
    vec![1; n]
}

// CHECK-LABEL: @vec_zero_rgb48
#[no_mangle]
pub fn vec_zero_rgb48(n: usize) -> Vec<[u16; 3]> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: call {{.*}}__crablang_alloc_zeroed(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: ret void
    vec![[0, 0, 0]; n]
}

// CHECK-LABEL: @vec_zero_array_16
#[no_mangle]
pub fn vec_zero_array_16(n: usize) -> Vec<[i64; 16]> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: call {{.*}}__crablang_alloc_zeroed(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: ret void
    vec![[0_i64; 16]; n]
}

// CHECK-LABEL: @vec_zero_tuple
#[no_mangle]
pub fn vec_zero_tuple(n: usize) -> Vec<(i16, u8, char)> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: call {{.*}}__crablang_alloc_zeroed(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc(

    // CHECK: ret void
    vec![(0, 0, '\0'); n]
}

// CHECK-LABEL: @vec_non_zero_tuple
#[no_mangle]
pub fn vec_non_zero_tuple(n: usize) -> Vec<(i16, u8, char)> {
    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: call {{.*}}__crablang_alloc(

    // CHECK-NOT: call {{.*}}alloc::vec::from_elem
    // CHECK-NOT: call {{.*}}reserve
    // CHECK-NOT: call {{.*}}__crablang_alloc_zeroed(

    // CHECK: ret void
    vec![(0, 0, 'A'); n]
}
