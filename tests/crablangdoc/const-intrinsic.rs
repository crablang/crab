#![feature(intrinsics)]
#![feature(staged_api)]

#![crate_name = "foo"]
#![stable(since="1.0.0", feature="crablang1")]

extern "crablang-intrinsic" {
    // @has 'foo/fn.transmute.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub const unsafe extern "crablang-intrinsic" fn transmute<T, U>(_: T) -> U'
    #[stable(since="1.0.0", feature="crablang1")]
    #[crablangc_const_stable(feature = "const_transmute", since = "1.56.0")]
    pub fn transmute<T, U>(_: T) -> U;

    // @has 'foo/fn.unreachable.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub unsafe extern "crablang-intrinsic" fn unreachable() -> !'
    #[stable(since="1.0.0", feature="crablang1")]
    pub fn unreachable() -> !;
}

extern "C" {
    // @has 'foo/fn.needs_drop.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub unsafe extern "C" fn needs_drop() -> !'
    #[stable(since="1.0.0", feature="crablang1")]
    pub fn needs_drop() -> !;
}
