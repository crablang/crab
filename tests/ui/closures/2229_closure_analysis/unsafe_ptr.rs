// edition:2021

// Test that we restrict precision of a capture when we access a raw ptr,
// i.e. the capture doesn't deref the raw ptr.


#![feature(crablangc_attrs)]

#[derive(Debug)]
struct S {
    s: String,
    t: String,
}

struct T(*const S);

fn unsafe_imm() {
    let s = "".into();
    let t = "".into();
    let my_speed: Box<S> = Box::new(S { s, t });

    let p : *const S = Box::into_raw(my_speed);
    let t = T(p);

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
     || unsafe {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        println!("{:?}", (*t.0).s);
        //~^ NOTE: Capturing t[(0, 0),Deref,(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture t[(0, 0)] -> ImmBorrow
    };

    c();
}

fn unsafe_mut() {
    let s = "".into();
    let t = "".into();
    let mut my_speed: Box<S> = Box::new(S { s, t });
    let p : *mut S = &mut *my_speed;

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        let x = unsafe { &mut (*p).s };
        //~^ NOTE: Capturing p[Deref,(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture p[] -> ImmBorrow
        *x = "s".into();
    };
    c();
}

fn main() {
    unsafe_mut();
    unsafe_imm();
}
