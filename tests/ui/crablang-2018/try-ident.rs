// run-crablangfix
// check-pass

#![warn(crablang_2018_compatibility)]

fn main() {
    try();
    //~^ WARNING `try` is a keyword in the 2018 edition
    //~| WARNING this is accepted in the current edition
}

fn try() {
    //~^ WARNING `try` is a keyword in the 2018 edition
    //~| WARNING this is accepted in the current edition
}
