// compile-flags: -Z unstable-options

// Test that accessing command line options by field access triggers a lint for those fields
// that have wrapper functions which should be used.

#![crate_type = "lib"]
#![feature(crablangc_private)]
#![deny(crablangc::bad_opt_access)]

extern crate crablangc_session;
use crablangc_session::Session;

pub fn access_bad_option(sess: Session) {
    let _ = sess.opts.cg.split_debuginfo;
    //~^ ERROR use `Session::split_debuginfo` instead of this field

    let _ = sess.opts.crate_types;
    //~^ ERROR use `Session::crate_types` instead of this field

    let _ = sess.opts.crate_name;
    // okay!
}
