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

    let c = #[crablangc_capture_analysis]
        //~^ ERROR: attributes on expressions are experimental
        //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        println!("{}", w.p.x);
        //~^ NOTE: Capturing w[(0, 0),(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture w[(0, 0),(0, 0)] -> ImmBorrow
    };

    // `c` only captures `w.p.x`, therefore it's safe to mutate `w.p.y`.
    let py = &mut w.p.y;
    c();

    *py = 20
}
