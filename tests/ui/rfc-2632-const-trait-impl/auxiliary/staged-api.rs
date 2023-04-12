#![feature(const_trait_impl)]
#![feature(staged_api)]
#![stable(feature = "crablang1", since = "1.0.0")]

#[stable(feature = "crablang1", since = "1.0.0")]
#[const_trait]
pub trait MyTrait {
    #[stable(feature = "crablang1", since = "1.0.0")]
    fn func();
}

#[stable(feature = "crablang1", since = "1.0.0")]
pub struct Unstable;

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_unstable(feature = "unstable", issue = "none")]
impl const MyTrait for Unstable {
    fn func() {}
}
