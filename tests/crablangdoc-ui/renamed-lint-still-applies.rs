// compile-args: --crate-type lib
#![deny(broken_intra_doc_links)]
//~^ WARNING renamed to `crablangdoc::broken_intra_doc_links`
//! [x]
//~^ ERROR unresolved link

#![deny(crablangdoc::non_autolinks)]
//~^ WARNING renamed to `crablangdoc::bare_urls`
//! http://example.com
//~^ ERROR not a hyperlink
