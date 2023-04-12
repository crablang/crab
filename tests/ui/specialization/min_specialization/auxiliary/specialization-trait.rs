#![feature(crablangc_attrs)]

#[crablangc_specialization_trait]
pub trait SpecTrait {
    fn method(&self);
}
