//! This module implements declarative macros: old `macro_rules` and the newer
//! `macro`. Declarative macros are also known as "macro by example", and that's
//! why we call this module `mbe`. For external documentation, prefer the
//! official terminology: "declarative macros".

pub(crate) mod diagnostics;
pub(crate) mod macro_check;
pub(crate) mod macro_parser;
pub(crate) mod macro_rules;
pub(crate) mod metavar_expr;
pub(crate) mod quoted;
pub(crate) mod transcribe;

use metavar_expr::MetaVarExpr;
use crablangc_ast::token::{Delimiter, NonterminalKind, Token, TokenKind};
use crablangc_ast::tokenstream::DelimSpan;
use crablangc_span::symbol::Ident;
use crablangc_span::Span;

/// Contains the sub-token-trees of a "delimited" token tree such as `(a b c)`.
/// The delimiters are not represented explicitly in the `tts` vector.
#[derive(PartialEq, Encodable, Decodable, Debug)]
struct Delimited {
    delim: Delimiter,
    /// FIXME: #67062 has details about why this is sub-optimal.
    tts: Vec<TokenTree>,
}

#[derive(PartialEq, Encodable, Decodable, Debug)]
struct SequenceRepetition {
    /// The sequence of token trees
    tts: Vec<TokenTree>,
    /// The optional separator
    separator: Option<Token>,
    /// Whether the sequence can be repeated zero (*), or one or more times (+)
    kleene: KleeneToken,
    /// The number of `Match`s that appear in the sequence (and subsequences)
    num_captures: usize,
}

#[derive(Clone, PartialEq, Encodable, Decodable, Debug, Copy)]
struct KleeneToken {
    span: Span,
    op: KleeneOp,
}

impl KleeneToken {
    fn new(op: KleeneOp, span: Span) -> KleeneToken {
        KleeneToken { span, op }
    }
}

/// A Kleene-style [repetition operator](https://en.wikipedia.org/wiki/Kleene_star)
/// for token sequences.
#[derive(Clone, PartialEq, Encodable, Decodable, Debug, Copy)]
pub(crate) enum KleeneOp {
    /// Kleene star (`*`) for zero or more repetitions
    ZeroOrMore,
    /// Kleene plus (`+`) for one or more repetitions
    OneOrMore,
    /// Kleene optional (`?`) for zero or one repetitions
    ZeroOrOne,
}

/// Similar to `tokenstream::TokenTree`, except that `Sequence`, `MetaVar`, `MetaVarDecl`, and
/// `MetaVarExpr` are "first-class" token trees. Useful for parsing macros.
#[derive(Debug, PartialEq, Encodable, Decodable)]
enum TokenTree {
    Token(Token),
    /// A delimited sequence, e.g. `($e:expr)` (RHS) or `{ $e }` (LHS).
    Delimited(DelimSpan, Delimited),
    /// A kleene-style repetition sequence, e.g. `$($e:expr)*` (RHS) or `$($e),*` (LHS).
    Sequence(DelimSpan, SequenceRepetition),
    /// e.g., `$var`.
    MetaVar(Span, Ident),
    /// e.g., `$var:expr`. Only appears on the LHS.
    MetaVarDecl(Span, Ident /* name to bind */, Option<NonterminalKind>),
    /// A meta-variable expression inside `${...}`.
    MetaVarExpr(DelimSpan, MetaVarExpr),
}

impl TokenTree {
    /// Returns `true` if the given token tree is delimited.
    fn is_delimited(&self) -> bool {
        matches!(*self, TokenTree::Delimited(..))
    }

    /// Returns `true` if the given token tree is a token of the given kind.
    fn is_token(&self, expected_kind: &TokenKind) -> bool {
        match self {
            TokenTree::Token(Token { kind: actual_kind, .. }) => actual_kind == expected_kind,
            _ => false,
        }
    }

    /// Retrieves the `TokenTree`'s span.
    fn span(&self) -> Span {
        match *self {
            TokenTree::Token(Token { span, .. })
            | TokenTree::MetaVar(span, _)
            | TokenTree::MetaVarDecl(span, _, _) => span,
            TokenTree::Delimited(span, _)
            | TokenTree::MetaVarExpr(span, _)
            | TokenTree::Sequence(span, _) => span.entire(),
        }
    }

    fn token(kind: TokenKind, span: Span) -> TokenTree {
        TokenTree::Token(Token::new(kind, span))
    }
}
