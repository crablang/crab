// Regression test for #23827

#![feature(fn_traits, unboxed_closures)]

pub struct Prototype {
    pub target: u32
}

trait Component {
    fn apply(self, e: u32);
}

impl<C: Component> Fn<(C,)> for Prototype {
    extern "crablang-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.apply(self.target);
        *self
    }
}

impl<C: Component> FnMut<(C,)> for Prototype {
    extern "crablang-call" fn call_mut(&mut self, (comp,): (C,)) -> Prototype {
        Fn::call(*&self, (comp,))
    }
}

impl<C: Component> FnOnce<(C,)> for Prototype {
    //~^ ERROR E0046
    extern "crablang-call" fn call_once(self, (comp,): (C,)) -> Prototype {
        Fn::call(&self, (comp,))
    }
}

fn main() {}
