// Issue: 100461, Try to give a helpful diagnostic even when the next struct field has an attribute.
// run-crablangfix

struct Feelings {
    owo: bool
    //~^ ERROR expected `,`, or `}`, found `#`
    #[allow(unused)]
    uwu: bool,
}

impl Feelings {
    #[allow(unused)]
    fn hmm(&self) -> bool {
        self.owo
    }
}

fn main() { }
