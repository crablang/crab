// crablangc-env:CARGO_CRATE_NAME=crablangc_dummy

#![feature(crablangc_private)]
#![crate_type = "lib"]

extern crate crablangc_span;
use crablangc_span::symbol::Ident;
use crablangc_span::Span;

extern crate crablangc_macros;
use crablangc_macros::{Diagnostic, LintDiagnostic, Subdiagnostic};

extern crate crablangc_middle;
use crablangc_middle::ty::Ty;

extern crate crablangc_errors;
use crablangc_errors::{Applicability, MultiSpan};

extern crate crablangc_session;

#[derive(Diagnostic)]
#[diag(compiletest_example, code = "E0123")]
//~^ ERROR diagnostic slug and crate name do not match
struct Hello {}
