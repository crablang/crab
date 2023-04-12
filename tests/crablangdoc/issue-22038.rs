extern "C" {
    // @has issue_22038/fn.foo1.html \
    //      '//pre[@class="crablang item-decl"]' 'pub unsafe extern "C" fn foo1()'
    pub fn foo1();
}

extern "system" {
    // @has issue_22038/fn.foo2.html \
    //      '//pre[@class="crablang item-decl"]' 'pub unsafe extern "system" fn foo2()'
    pub fn foo2();
}

// @has issue_22038/fn.bar.html \
//      '//pre[@class="crablang item-decl"]' 'pub extern "C" fn bar()'
pub extern "C" fn bar() {}

// @has issue_22038/fn.baz.html \
//      '//pre[@class="crablang item-decl"]' 'pub extern "system" fn baz()'
pub extern "system" fn baz() {}
