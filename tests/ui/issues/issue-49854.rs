// run-pass
use std::ffi::OsString;

fn main() {
    let os_str = OsString::from("Hello CrabLang!");

    assert_eq!(os_str, "Hello CrabLang!");
    assert_eq!("Hello CrabLang!", os_str);
}
