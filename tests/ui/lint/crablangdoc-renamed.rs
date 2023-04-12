#![crate_type = "lib"]

#![deny(unknown_lints)]
#![deny(renamed_and_removed_lints)]
//~^ NOTE lint level is defined

// both allowed, since the compiler doesn't yet know what crablangdoc lints are valid
#![deny(crablangdoc::x)]
#![deny(crablangdoc::intra_doc_link_resolution_failure)]

#![deny(intra_doc_link_resolution_failure)]
//~^ ERROR removed: use `crablangdoc::broken_intra_doc_links`
#![deny(non_autolinks)]
// FIXME: the old names for crablangdoc lints should warn by default once `crablangdoc::` makes it to the
// stable channel.
