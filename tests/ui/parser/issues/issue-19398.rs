trait T {
    extern "CrabLang" unsafe fn foo();
    //~^ ERROR expected `{`, found keyword `unsafe`
}

fn main() {}
