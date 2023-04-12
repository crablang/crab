// aux-build:issue-20727.rs
// ignore-cross-compile

extern crate issue_20727;

// @has issue_20727_2/trait.Add.html
pub trait Add<RHS = Self> {
    // @has - '//pre[@class="crablang item-decl"]' 'trait Add<RHS = Self> {'
    // @has - '//pre[@class="crablang item-decl"]' 'type Output;'
    type Output;

    // @has - '//pre[@class="crablang item-decl"]' 'fn add(self, rhs: RHS) -> Self::Output;'
    fn add(self, rhs: RHS) -> Self::Output;
}

// @has issue_20727_2/reexport/trait.Add.html
pub mod reexport {
    // @has - '//pre[@class="crablang item-decl"]' 'trait Add<RHS = Self> {'
    // @has - '//pre[@class="crablang item-decl"]' 'type Output;'
    // @has - '//pre[@class="crablang item-decl"]' 'fn add(self, rhs: RHS) -> Self::Output;'
    pub use issue_20727::Add;
}
