// run-crablangfix

#![deny(unused_parens)]
#![allow(while_true)] // for crablangfix

#[derive(Eq, PartialEq)]
struct X { y: bool }
impl X {
    fn foo(&self, conjunct: bool) -> bool { self.y && conjunct }
}

fn foo() -> isize {
    return (1); //~ ERROR unnecessary parentheses around `return` value
}
fn bar(y: bool) -> X {
    return (X { y }); //~ ERROR unnecessary parentheses around `return` value
}

pub fn unused_parens_around_return_type() -> (u32) { //~ ERROR unnecessary parentheses around type
    panic!()
}

pub fn unused_parens_around_block_return() -> u32 {
    let _foo = {
        (5) //~ ERROR unnecessary parentheses around block return value
    };
    (5) //~ ERROR unnecessary parentheses around block return value
}

pub trait Trait {
    fn test(&self);
}

pub fn passes_unused_parens_lint() -> &'static (dyn Trait) {
    panic!()
}

macro_rules! baz {
    ($($foo:expr),+) => {
        ($($foo),*)
    }
}

pub const CONST_ITEM: usize = (10); //~ ERROR unnecessary parentheses around assigned value
pub static STATIC_ITEM: usize = (10); //~ ERROR unnecessary parentheses around assigned value

fn main() {
    foo();
    bar((true)); //~ ERROR unnecessary parentheses around function argument

    if (true) {} //~ ERROR unnecessary parentheses around `if` condition
    while (true) {} //~ ERROR unnecessary parentheses around `while` condition
    match (true) { //~ ERROR unnecessary parentheses around `match` scrutinee expression
        _ => {}
    }
    if let 1 = (1) {} //~ ERROR unnecessary parentheses around `let` scrutinee expression
    while let 1 = (2) {} //~ ERROR unnecessary parentheses around `let` scrutinee expression
    let v = X { y: false };
    // struct lits needs parens, so these shouldn't warn.
    if (v == X { y: true }) {}
    if (X { y: true } == v) {}
    if (X { y: false }.y) {}
    // this shouldn't warn, because the parens are necessary to disambiguate let chains
    if let true = (true && false) {}

    while (X { y: false }.foo(true)) {}
    while (true | X { y: false }.y) {}

    match (X { y: false }) {
        _ => {}
    }

    X { y: false }.foo((true)); //~ ERROR unnecessary parentheses around method argument

    let mut _a = (0); //~ ERROR unnecessary parentheses around assigned value
    _a = (0); //~ ERROR unnecessary parentheses around assigned value
    _a += (1); //~ ERROR unnecessary parentheses around assigned value

    let _a = baz!(3, 4);
    let _b = baz!(3);
}
