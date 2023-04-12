extern "crablang-intrinsic" {   //~ ERROR intrinsics are subject to change
    fn bar(); //~ ERROR unrecognized intrinsic function: `bar`
}

extern "crablang-intrinsic" fn baz() {} //~ ERROR intrinsics are subject to change
//~^ ERROR intrinsic must be in

fn main() {}
