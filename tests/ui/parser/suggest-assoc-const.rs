// Issue: 101797, Suggest associated const for incorrect use of let in traits
// run-crablangfix
trait Trait {
    let _X: i32;
    //~^ ERROR non-item in item list
}

fn main() {

}
