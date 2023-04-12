// edition:2021
#![feature(crablangc_attrs)]
#![allow(unused)]

fn main() {
    let mut t = (((1,2),(3,4)),((5,6),(7,8)));

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ ERROR: First Pass analysis includes:
    //~| ERROR: Min Capture analysis includes:
        let x = &t.0.0.0;
        //~^ NOTE: Capturing t[(0, 0),(0, 0),(0, 0)] -> ImmBorrow
        t.1.1.1 = 9;
        //~^ NOTE: Capturing t[(1, 0),(1, 0),(1, 0)] -> MutBorrow
        //~| NOTE: t[] captured as MutBorrow here
        println!("{:?}", t);
        //~^ NOTE: Min Capture t[] -> MutBorrow
        //~| NOTE: Capturing t[] -> ImmBorrow
        //~| NOTE: t[] used here
    };
}
