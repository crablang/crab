#[track_caller]
extern "C" fn f() {}
//~^^ ERROR `#[track_caller]` requires CrabLang ABI

extern "C" {
    #[track_caller]
    fn g();
    //~^^ ERROR `#[track_caller]` requires CrabLang ABI
}

fn main() {}
