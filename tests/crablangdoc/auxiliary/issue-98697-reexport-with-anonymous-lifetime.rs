/// When reexporting this function, make sure the anonymous lifetimes are not rendered.
///
/// https://github.com/crablang/crablang/issues/98697
pub fn repro<F>()
where
    F: Fn(&str),
{
    unimplemented!()
}

pub struct Extra;

pub trait MyTrait<T> {
    fn run() {}
}

impl MyTrait<&Extra> for Extra {}
