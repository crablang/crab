mod diagnostic;
mod diagnostic_builder;
mod error;
mod fluent;
mod subdiagnostic;
mod utils;

use diagnostic::{DiagnosticDerive, LintDiagnosticDerive};
pub(crate) use fluent::fluent_messages;
use proc_macro2::TokenStream;
use quote::format_ident;
use subdiagnostic::SubdiagnosticDeriveBuilder;
use synstructure::Structure;

/// Implements `#[derive(Diagnostic)]`, which allows for errors to be specified as a struct,
/// independent from the actual diagnostics emitting code.
///
/// ```ignore (crablang)
/// # extern crate crablangc_errors;
/// # use crablangc_errors::Applicability;
/// # extern crate crablangc_span;
/// # use crablangc_span::{symbol::Ident, Span};
/// # extern crate crablang_middle;
/// # use crablangc_middle::ty::Ty;
/// #[derive(Diagnostic)]
/// #[diag(borrowck_move_out_of_borrow, code = "E0505")]
/// pub struct MoveOutOfBorrowError<'tcx> {
///     pub name: Ident,
///     pub ty: Ty<'tcx>,
///     #[primary_span]
///     #[label]
///     pub span: Span,
///     #[label(first_borrow_label)]
///     pub first_borrow_span: Span,
///     #[suggestion(code = "{name}.clone()")]
///     pub clone_sugg: Option<(Span, Applicability)>
/// }
/// ```
///
/// ```fluent
/// move_out_of_borrow = cannot move out of {$name} because it is borrowed
///     .label = cannot move out of borrow
///     .first_borrow_label = `{$ty}` first borrowed here
///     .suggestion = consider cloning here
/// ```
///
/// Then, later, to emit the error:
///
/// ```ignore (crablang)
/// sess.emit_err(MoveOutOfBorrowError {
///     expected,
///     actual,
///     span,
///     first_borrow_span,
///     clone_sugg: Some(suggestion, Applicability::MachineApplicable),
/// });
/// ```
///
/// See crablangc dev guide for more examples on using the `#[derive(Diagnostic)]`:
/// <https://crablangc-dev-guide.crablang.org/diagnostics/diagnostic-structs.html>
pub fn session_diagnostic_derive(s: Structure<'_>) -> TokenStream {
    DiagnosticDerive::new(format_ident!("diag"), format_ident!("handler"), s).into_tokens()
}

/// Implements `#[derive(LintDiagnostic)]`, which allows for lints to be specified as a struct,
/// independent from the actual lint emitting code.
///
/// ```ignore (crablang)
/// #[derive(LintDiagnostic)]
/// #[diag(lint_atomic_ordering_invalid_fail_success)]
/// pub struct AtomicOrderingInvalidLint {
///     method: Symbol,
///     success_ordering: Symbol,
///     fail_ordering: Symbol,
///     #[label(fail_label)]
///     fail_order_arg_span: Span,
///     #[label(success_label)]
///     #[suggestion(
///         code = "std::sync::atomic::Ordering::{success_suggestion}",
///         applicability = "maybe-incorrect"
///     )]
///     success_order_arg_span: Span,
/// }
/// ```
///
/// ```fluent
/// lint_atomic_ordering_invalid_fail_success = `{$method}`'s success ordering must be at least as strong as its failure ordering
///     .fail_label = `{$fail_ordering}` failure ordering
///     .success_label = `{$success_ordering}` success ordering
///     .suggestion = consider using `{$success_suggestion}` success ordering instead
/// ```
///
/// Then, later, to emit the error:
///
/// ```ignore (crablang)
/// cx.struct_span_lint(INVALID_ATOMIC_ORDERING, fail_order_arg_span, AtomicOrderingInvalidLint {
///     method,
///     success_ordering,
///     fail_ordering,
///     fail_order_arg_span,
///     success_order_arg_span,
/// });
/// ```
///
/// See crablangc dev guide for more examples on using the `#[derive(LintDiagnostic)]`:
/// <https://crablangc-dev-guide.crablang.org/diagnostics/diagnostic-structs.html#reference>
pub fn lint_diagnostic_derive(s: Structure<'_>) -> TokenStream {
    LintDiagnosticDerive::new(format_ident!("diag"), s).into_tokens()
}

/// Implements `#[derive(Subdiagnostic)]`, which allows for labels, notes, helps and
/// suggestions to be specified as a structs or enums, independent from the actual diagnostics
/// emitting code or diagnostic derives.
///
/// ```ignore (crablang)
/// #[derive(Subdiagnostic)]
/// pub enum ExpectedIdentifierLabel<'tcx> {
///     #[label(expected_identifier)]
///     WithoutFound {
///         #[primary_span]
///         span: Span,
///     }
///     #[label(expected_identifier_found)]
///     WithFound {
///         #[primary_span]
///         span: Span,
///         found: String,
///     }
/// }
///
/// #[derive(Subdiagnostic)]
/// #[suggestion(style = "verbose",parser::raw_identifier)]
/// pub struct RawIdentifierSuggestion<'tcx> {
///     #[primary_span]
///     span: Span,
///     #[applicability]
///     applicability: Applicability,
///     ident: Ident,
/// }
/// ```
///
/// ```fluent
/// parser_expected_identifier = expected identifier
///
/// parser_expected_identifier-found = expected identifier, found {$found}
///
/// parser_raw_identifier = escape `{$ident}` to use it as an identifier
/// ```
///
/// Then, later, to add the subdiagnostic:
///
/// ```ignore (crablang)
/// diag.subdiagnostic(ExpectedIdentifierLabel::WithoutFound { span });
///
/// diag.subdiagnostic(RawIdentifierSuggestion { span, applicability, ident });
/// ```
pub fn session_subdiagnostic_derive(s: Structure<'_>) -> TokenStream {
    SubdiagnosticDeriveBuilder::new().into_tokens(s)
}
