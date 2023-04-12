// We should not see the unused_attributes lint fire for
// crablangc_on_unimplemented, but with this bug we are seeing it fire (on
// subsequent runs) if incremental compilation is enabled.

// revisions: cfail1 cfail2
// build-pass (FIXME(62277): could be check-pass?)

#![feature(crablangc_attrs)]
#![deny(unused_attributes)]

#[crablangc_on_unimplemented = "invalid"]
trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

#[crablangc_on_unimplemented = "a usize is required to index into a slice"]
impl Index<usize> for [i32] {
    type Output = i32;
    fn index(&self, index: usize) -> &i32 {
        &self[index]
    }
}

fn main() {
    Index::<usize>::index(&[1, 2, 3] as &[i32], 2);
}
