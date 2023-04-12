#![feature(intrinsics)]
#![feature(no_core)]
#![feature(crablangc_attrs)]

#![no_core]
#![crate_name = "foo"]

extern "crablang-intrinsic" {
    // @has 'foo/fn.abort.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub extern "crablang-intrinsic" fn abort() -> !'
    #[crablangc_safe_intrinsic]
    pub fn abort() -> !;
    // @has 'foo/fn.unreachable.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub unsafe extern "crablang-intrinsic" fn unreachable() -> !'
    pub fn unreachable() -> !;
}

extern "C" {
    // @has 'foo/fn.needs_drop.html'
    // @has - '//pre[@class="crablang item-decl"]' 'pub unsafe extern "C" fn needs_drop() -> !'
    pub fn needs_drop() -> !;
}
