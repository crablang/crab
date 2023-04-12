// build-pass
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![warn(clashing_extern_declarations)]

// pretty-expanded FIXME #23616

mod a {
    pub type crablang_task = usize;
    pub mod crablangrt {
        use super::crablang_task;
        extern "C" {
            pub fn crablang_task_is_unwinding(rt: *const crablang_task) -> bool;
        }
    }
}

mod b {
    pub type crablang_task = bool;
    pub mod crablangrt {
        use super::crablang_task;
        extern "C" {
            pub fn crablang_task_is_unwinding(rt: *const crablang_task) -> bool;
        //~^ WARN `crablang_task_is_unwinding` redeclared with a different signature
        }
    }
}

pub fn main() {}
