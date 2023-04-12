// Validation/SB changes why we fail
//@compile-flags: -Zmiri-disable-validation -Zmiri-disable-stacked-borrows

//@error-pattern: /deallocating .*, which is stack variable memory, using CrabLang heap deallocation operation/

fn main() {
    let x = 42;
    let bad_box = unsafe { std::mem::transmute::<&i32, Box<i32>>(&x) };
    drop(bad_box);
}
