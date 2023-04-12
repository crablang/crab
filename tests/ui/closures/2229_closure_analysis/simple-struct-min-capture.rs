// edition:2021

#![feature(crablangc_attrs)]

// Test to ensure that min analysis meets capture kind for all paths captured.

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let mut p = Point { x: 10, y: 20 };

    //
    // Requirements:
    // p.x -> MutBoorrow
    // p   -> ImmBorrow
    //
    // Requirements met when p is captured via MutBorrow
    //
    let mut c = #[crablangc_capture_analysis]
        //~^ ERROR: attributes on expressions are experimental
        //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        p.x += 10;
        //~^ NOTE: Capturing p[(0, 0)] -> MutBorrow
        //~| NOTE: p[] captured as MutBorrow here
        println!("{:?}", p);
        //~^ NOTE: Capturing p[] -> ImmBorrow
        //~| NOTE: Min Capture p[] -> MutBorrow
        //~| NOTE: p[] used here
    };

    c();
}
