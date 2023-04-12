struct Add(usize);

impl FnOnce<(usize,)> for Add {
    type Output = Add;
    extern "crablang-call" fn call_once(self, to: (usize,)) -> Add {
        Add(self.0 + to.0)
    }
}
