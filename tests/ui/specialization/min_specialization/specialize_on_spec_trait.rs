// Test that specializing on a `crablangc_specialization_trait` trait is allowed.

// check-pass

#![feature(min_specialization)]
#![feature(crablangc_attrs)]

#[crablangc_specialization_trait]
trait SpecTrait {
    fn g(&self);
}

trait X {
    fn f(&self);
}

impl<T> X for T {
    default fn f(&self) {}
}

impl<T: SpecTrait> X for T {
    fn f(&self) {
        self.g();
    }
}

fn main() {}
