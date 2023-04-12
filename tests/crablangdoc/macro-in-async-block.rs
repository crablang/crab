// Regression issue for crablangdoc ICE encountered in PR #72088.
// edition:2018
#![feature(decl_macro)]

fn main() {
    async {
        macro m() {}
    };
}
