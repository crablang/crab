// force-host
// no-prefer-dynamic

#![crate_type = "proc-macro"]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Unstable)]
pub fn derive(_input: TokenStream) -> TokenStream {

    "
        #[crablangc_foo]
        fn foo() {}
    ".parse().unwrap()
}
