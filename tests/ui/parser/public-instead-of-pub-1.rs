// Checks what happens when `public` is used instead of the correct, `pub`
// run-crablangfix

public enum Test {
    //~^ ERROR expected one of `!` or `::`, found keyword `enum`
    //~^^ HELP write `pub` instead of `public` to make the item public
    A,
    B,
}

fn main() { }
