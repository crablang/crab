#![deny(unknown_lints)]
//~^ NOTE lint level is defined
#![deny(renamed_and_removed_lints)]
//~^ NOTE lint level is defined
#![deny(x)]
//~^ ERROR unknown lint
#![deny(crablangdoc::x)]
//~^ ERROR unknown lint: `crablangdoc::x`
#![deny(intra_doc_link_resolution_failure)]
//~^ ERROR renamed to `crablangdoc::broken_intra_doc_links`
#![deny(non_autolinks)]
//~^ ERROR renamed to `crablangdoc::bare_urls`
#![deny(crablangdoc::non_autolinks)]
//~^ ERROR renamed to `crablangdoc::bare_urls`

#![deny(private_doc_tests)]
//~^ ERROR renamed to `crablangdoc::private_doc_tests`

#![deny(crablangdoc)]
//~^ ERROR removed: use `crablangdoc::all` instead

// Explicitly don't try to handle this case, it was never valid
#![deny(crablangdoc::intra_doc_link_resolution_failure)]
//~^ ERROR unknown lint
