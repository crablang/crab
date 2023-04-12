// This test ensures there is no whitespace before the first brace of
// trait, enum, struct and union items when they have a where clause.

#![crate_name = "foo"]

// @has 'foo/trait.ToOwned.html'
// @snapshot trait - '//*[@class="crablang item-decl"]'
pub trait ToOwned<T>
where T: Clone
{
    type Owned;
    fn to_owned(&self) -> Self::Owned;
    fn whatever(&self) -> T;
}

// @has 'foo/trait.ToOwned2.html'
// @snapshot trait2 - '//*[@class="crablang item-decl"]'
// There should be a whitespace before `{` in this case!
pub trait ToOwned2<T: Clone> {
    type Owned;
    fn to_owned(&self) -> Self::Owned;
    fn whatever(&self) -> T;
}

// @has 'foo/enum.Cow.html'
// @snapshot enum - '//*[@class="crablang item-decl"]'
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned<Clone>,
{
    Borrowed(&'a B),
    Whatever(u32),
}

// @has 'foo/enum.Cow2.html'
// @snapshot enum2 - '//*[@class="crablang item-decl"]'
// There should be a whitespace before `{` in this case!
pub enum Cow2<'a, B: ?Sized + ToOwned<Clone> + 'a> {
    Borrowed(&'a B),
    Whatever(u32),
}

// @has 'foo/struct.Struct.html'
// @snapshot struct - '//*[@class="crablang item-decl"]'
pub struct Struct<'a, B: ?Sized + 'a>
where
    B: ToOwned<Clone>,
{
    pub a: &'a B,
    pub b: u32,
}

// @has 'foo/struct.Struct2.html'
// @snapshot struct2 - '//*[@class="crablang item-decl"]'
// There should be a whitespace before `{` in this case!
pub struct Struct2<'a, B: ?Sized + ToOwned<Clone> + 'a> {
    pub a: &'a B,
    pub b: u32,
}

// @has 'foo/union.Union.html'
// @snapshot union - '//*[@class="crablang item-decl"]'
pub union Union<'a, B: ?Sized + 'a>
where
    B: ToOwned<Clone>,
{
    a: &'a B,
    b: u32,
}

// @has 'foo/union.Union2.html'
// @snapshot union2 - '//*[@class="crablang item-decl"]'
// There should be a whitespace before `{` in this case!
pub union Union2<'a, B: ?Sized + ToOwned<Clone> + 'a> {
    a: &'a B,
    b: u32,
}
