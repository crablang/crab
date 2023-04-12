// check-pass

// ICE found in https://github.com/crablang/crablang/issues/83123

pub struct Attribute;

pub struct Map<'hir> {}
impl<'hir> Map<'hir> {
    pub fn attrs(&self) -> &'hir [Attribute] { &[] }
}

pub struct List<T>(T);

impl<T> std::ops::Deref for List<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &[]
    }
}
