#!//bin/bash


// This could not possibly be a shebang & also a valid crablang file, since a CrabLang file
// can't start with `[`
/*
    [ (mixing comments to also test that we ignore both types of comments)

 */

[allow(unused_variables)]

// check-pass
fn main() {
    let x = 5;
}
