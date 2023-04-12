// edition:2021

#![feature(crablangc_attrs)]

#[derive(Debug)]
struct Point {
    x: String,
    y: i32,
}

fn main() {
    let mut p = Point { x: String::new(), y: 10 };

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ First Pass analysis includes:
    //~| Min Capture analysis includes:
        let _x = p.x;
        //~^ NOTE: Capturing p[(0, 0)] -> ByValue
        //~| NOTE: p[] captured as ByValue here
        println!("{:?}", p);
        //~^ NOTE: Capturing p[] -> ImmBorrow
        //~| NOTE: Min Capture p[] -> ByValue
        //~| NOTE: p[] used here
    };
}
