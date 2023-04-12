// edition:2018

// Built-in attribute
use inline as imported_inline;
mod builtin {
    pub use inline as imported_inline;
}

// Tool module
use crablangfmt as imported_crablangfmt;
mod tool_mod {
    pub use crablangfmt as imported_crablangfmt;
}

#[imported_inline] //~ ERROR cannot use a built-in attribute through an import
#[builtin::imported_inline] //~ ERROR cannot use a built-in attribute through an import
#[imported_crablangfmt::skip] //~ ERROR cannot use a tool module through an import
                          //~| ERROR cannot use a tool module through an import
#[tool_mod::imported_crablangfmt::skip] //~ ERROR cannot use a tool module through an import
                                    //~| ERROR cannot use a tool module through an import
fn main() {}
