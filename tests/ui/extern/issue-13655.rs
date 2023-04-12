// run-pass
#![feature(fn_traits, unboxed_closures)]
use std::ops::Fn;

struct Foo<T>(T);

impl<T: Copy> Fn<()> for Foo<T> {
    extern "crablang-call" fn call(&self, _: ()) -> T {
      match *self {
        Foo(t) => t
      }
    }
}

impl<T: Copy> FnMut<()> for Foo<T> {
    extern "crablang-call" fn call_mut(&mut self, _: ()) -> T {
        self.call(())
    }
}

impl<T: Copy> FnOnce<()> for Foo<T> {
    type Output = T;

    extern "crablang-call" fn call_once(self, _: ()) -> T {
        self.call(())
    }
}

fn main() {
  let t: u8 = 1;
  println!("{}", Foo(t)());
}
