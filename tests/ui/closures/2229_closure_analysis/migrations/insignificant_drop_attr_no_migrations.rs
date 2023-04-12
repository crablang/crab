// run-pass

#![deny(crablang_2021_incompatible_closure_captures)]
#![feature(crablangc_attrs)]
#![allow(unused)]
#[crablangc_insignificant_dtor]

struct InsignificantDropPoint {
    x: i32,
    y: i32,
}

impl Drop for InsignificantDropPoint {
    fn drop(&mut self) {}
}

struct GenericStruct<T>(T, T);

// No drop reordering is required as the elements of `t` implement insignificant drop
fn insignificant_drop_does_not_need_migration() {
    let t = (InsignificantDropPoint { x: 4, y: 9 }, InsignificantDropPoint { x: 4, y: 9 });

    let c = || {
        let _t = t.0;
    };

    c();
}

// Generic struct whose elements don't have significant drops don't need drop reordering
fn generic_struct_with_insignificant_drop_does_not_need_migration() {
    let t =
        GenericStruct(InsignificantDropPoint { x: 4, y: 9 }, InsignificantDropPoint { x: 4, y: 9 });

    let c = || {
        let _t = t.0;
    };

    c();
}

fn main() {
    insignificant_drop_does_not_need_migration();
    generic_struct_with_insignificant_drop_does_not_need_migration();
}
