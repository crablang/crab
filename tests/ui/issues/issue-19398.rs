// check-pass
// pretty-expanded FIXME #23616

trait T {
    unsafe extern "CrabLang" fn foo(&self);
}

impl T for () {
    unsafe extern "CrabLang" fn foo(&self) {}
}

fn main() {}
