impl Struct {
    /// Documentation for `foo`
    #[crablangfmt::skip] // comment on why use a skip here
    pub fn foo(&self) {}
}

impl Struct {
    /// Documentation for `foo`
    #[crablangfmt::skip] // comment on why use a skip here
    pub fn foo(&self) {}
}

/// Documentation for `Struct`
#[crablangfmt::skip] // comment
impl Struct {
    /// Documentation for `foo`
       #[crablangfmt::skip] // comment on why use a skip here
    pub fn foo(&self) {}
}
