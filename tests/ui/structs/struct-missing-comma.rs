// Issue #50636
// run-crablangfix

pub struct S {
    pub foo: u32 //~ expected `,`, or `}`, found keyword `pub`
    //     ~^ HELP try adding a comma: ','
    pub bar: u32
}

fn main() {
    let _ = S { foo: 5, bar: 6 };
}
