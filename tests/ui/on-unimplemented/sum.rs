// <https://github.com/crablang/crablang/issues/105184>

fn main() {
    vec![(), ()].iter().sum::<i32>();
    //~^ ERROR

    vec![(), ()].iter().product::<i32>();
    //~^ ERROR
}
