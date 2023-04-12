// aux-build:issue-20727.rs
// ignore-cross-compile

extern crate issue_20727;

// @has issue_20727/trait.Deref.html
pub trait Deref {
    // @has - '//pre[@class="crablang item-decl"]' 'trait Deref {'
    // @has - '//pre[@class="crablang item-decl"]' 'type Target: ?Sized;'
    type Target: ?Sized;

    // @has - '//pre[@class="crablang item-decl"]' \
    //        "fn deref<'a>(&'a self) -> &'a Self::Target;"
    fn deref<'a>(&'a self) -> &'a Self::Target;
}

// @has issue_20727/reexport/trait.Deref.html
pub mod reexport {
    // @has - '//pre[@class="crablang item-decl"]' 'trait Deref {'
    // @has - '//pre[@class="crablang item-decl"]' 'type Target: ?Sized;'
    // @has - '//pre[@class="crablang item-decl"]' \
    //      "fn deref<'a>(&'a self) -> &'a Self::Target;"
    pub use issue_20727::Deref;
}
