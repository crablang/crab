#![feature(crablangc_attrs)]
#![feature(crablangdoc_internals)]
#![no_std]

pub mod str {
    #![crablangc_doc_primitive = "str"]

    impl str {
        // @hasraw search-index.js foo
        #[crablangc_allow_incoherent_impl]
        pub fn foo(&self) {}
    }
}
