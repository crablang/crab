// edition:2021

#![feature(crablangc_attrs)]
#![allow(unused)]

struct Point {
    x: i32,
    y: i32,
}
struct Wrapper {
    p: Point,
}

fn main() {
    let mut w = Wrapper { p: Point { x: 10, y: 10 } };

    // Only paths that appears within the closure that directly start off
    // a variable defined outside the closure are captured.
    //
    // Therefore `w.p` is captured
    // Note that `wp.x` doesn't start off a variable defined outside the closure.
    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        let wp = &w.p;
        //~^ NOTE: Capturing w[(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture w[(0, 0)] -> ImmBorrow
        println!("{}", wp.x);
    };

    // Since `c` captures `w.p` by an ImmBorrow, `w.p.y` can't be mutated.
    let py = &mut w.p.y;
    c();

    *py = 20
}
