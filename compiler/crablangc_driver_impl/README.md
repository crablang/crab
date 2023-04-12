The `driver` crate is effectively the "main" function for the crablang
compiler. It orchestrates the compilation process and "knits together"
the code from the other crates within crablangc. This crate itself does
not contain any of the "main logic" of the compiler (though it does
have some code related to pretty printing or other minor compiler
options).

For more information about how the driver works, see the [crablangc dev guide].

[crablangc dev guide]: https://crablangc-dev-guide.crablang.org/crablangc-driver.html
