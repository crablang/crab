#![feature(extern_types)]

extern {
    /// No inner information
    pub type Foo;
}

// @is "$.index[*][?(@.docs=='No inner information')].name" '"Foo"'
// @is "$.index[*][?(@.docs=='No inner information')].kind" '"foreign_type"'
// @!has "$.index[*][?(@.docs=='No inner information')].inner"
