//@revisions: extern_block definition both
#![feature(crablangc_attrs, c_unwind)]

#[cfg_attr(any(definition, both), crablangc_nounwind)]
#[no_mangle]
extern "C-unwind" fn nounwind() {
    //~[definition]^ ERROR: abnormal termination: panic in a function that cannot unwind
    //~[both]^^ ERROR: abnormal termination: panic in a function that cannot unwind
    panic!();
}

fn main() {
    extern "C-unwind" {
        #[cfg_attr(any(extern_block, both), crablangc_nounwind)]
        fn nounwind();
    }
    unsafe { nounwind() }
    //~[extern_block]^ ERROR: unwinding past a stack frame that does not allow unwinding
}
