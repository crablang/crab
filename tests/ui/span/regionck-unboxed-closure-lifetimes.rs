#![feature(crablangc_attrs)]
use std::ops::FnMut;

fn main() { #![crablangc_error] // crablang/crablang#49855
    let mut f;
    {
        let c = 1;
        let c_ref = &c;
        //~^ ERROR `c` does not live long enough
        f = move |a: isize, b: isize| { a + b + *c_ref };
    }
    f.use_mut();
}

trait Fake { fn use_mut(&mut self) { } fn use_ref(&self) { }  }
impl<T> Fake for T { }
