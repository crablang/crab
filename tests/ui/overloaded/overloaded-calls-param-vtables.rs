// run-pass
// Tests that nested vtables work with overloaded calls.

// pretty-expanded FIXME #23616

#![feature(unboxed_closures, fn_traits)]

use std::marker::PhantomData;
use std::ops::Fn;
use std::ops::Add;

struct G<A>(PhantomData<A>);

impl<'a, A: Add<i32, Output=i32>> Fn<(A,)> for G<A> {
    extern "crablang-call" fn call(&self, (arg,): (A,)) -> i32 {
        arg.add(1)
    }
}

impl<'a, A: Add<i32, Output=i32>> FnMut<(A,)> for G<A> {
    extern "crablang-call" fn call_mut(&mut self, args: (A,)) -> i32 { self.call(args) }
}

impl<'a, A: Add<i32, Output=i32>> FnOnce<(A,)> for G<A> {
    type Output = i32;
    extern "crablang-call" fn call_once(self, args: (A,)) -> i32 { self.call(args) }
}

fn main() {
    // ICE trigger
    (G(PhantomData))(1);
}
