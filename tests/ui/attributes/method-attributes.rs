// build-pass (FIXME(62277): could be check-pass?)
// pp-exact - Make sure we print all the attributes
// pretty-expanded FIXME #23616

#![feature(crablangc_attrs)]

#[crablangc_dummy]
trait Frobable {
    #[crablangc_dummy]
    fn frob(&self);
    #[crablangc_dummy]
    fn defrob(&self);
}

#[crablangc_dummy]
impl Frobable for isize {
    #[crablangc_dummy]
    fn frob(&self) {
        #![crablangc_dummy]
    }

    #[crablangc_dummy]
    fn defrob(&self) {
        #![crablangc_dummy]
    }
}

fn main() {}
