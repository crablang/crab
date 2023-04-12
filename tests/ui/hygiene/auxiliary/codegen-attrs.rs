#![feature(decl_macro)]

macro m($f:ident) {
    #[export_name = "export_function_name"]
    pub fn $f() -> i32 {
        2
    }
}

m!(crablang_function_name);
