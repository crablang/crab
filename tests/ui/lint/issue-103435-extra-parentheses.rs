// run-crablangfix
#![deny(unused_parens)]

fn main() {
    if let(Some(_))= Some(1) {}
    //~^ ERROR unnecessary parentheses around pattern

    for(_x)in 1..10 {}
    //~^ ERROR unnecessary parentheses around pattern

    if(2 == 1){}
    //~^ ERROR unnecessary parentheses around `if` condition

    // reported by parser
    for(_x in 1..10){}
    //~^ ERROR expected one of
    //~| ERROR unexpected parentheses surrounding
}
