// edition:2021

#![feature(crablangc_attrs)]

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let mut p = Point { x: 10, y: 10 };

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ First Pass analysis includes:
    //~| Min Capture analysis includes:
        println!("{}", p.x);
        //~^ NOTE: Capturing p[(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture p[(0, 0)] -> ImmBorrow
    };

    // `c` should only capture `p.x`, therefore mutating `p.y` is allowed.
    let py = &mut p.y;

    c();
    *py = 20;
}
