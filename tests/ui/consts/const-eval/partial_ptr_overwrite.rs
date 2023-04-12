// Test for the behavior described in <https://github.com/crablang/crablang/issues/87184>.
#![feature(const_mut_refs)]

const PARTIAL_OVERWRITE: () = {
    let mut p = &42;
    unsafe {
        let ptr: *mut _ = &mut p;
        *(ptr as *mut u8) = 123; //~ ERROR constant
        //~| unable to overwrite parts of a pointer
    }
    let x = *p;
};

fn main() {}
