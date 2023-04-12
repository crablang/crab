//! proc-macro server implementation
//!
//! Based on idea from <https://github.com/fedochet/crablang-proc-macro-expander>
//! The lib-proc-macro server backend is `TokenStream`-agnostic, such that
//! we could provide any TokenStream implementation.
//! The original idea from fedochet is using proc-macro2 as backend,
//! we use tt instead for better integration with RA.
//!
//! FIXME: No span and source file information is implemented yet

use super::proc_macro::{
    self,
    bridge::{self, server},
};

mod token_stream;
pub use token_stream::TokenStream;
use token_stream::TokenStreamBuilder;

mod symbol;
pub use symbol::*;

use std::ops::Bound;

use crate::tt;

type Group = tt::Subtree;
type TokenTree = tt::TokenTree;
type Punct = tt::Punct;
type Spacing = tt::Spacing;
type Literal = tt::Literal;
type Span = tt::TokenId;

#[derive(Clone)]
pub struct SourceFile {
    // FIXME stub
}

type Level = super::proc_macro::Level;
type LineColumn = super::proc_macro::LineColumn;

pub struct FreeFunctions;

#[derive(Default)]
pub struct CrabLangAnalyzer {
    // FIXME: store span information here.
}

impl server::Types for CrabLangAnalyzer {
    type FreeFunctions = FreeFunctions;
    type TokenStream = TokenStream;
    type SourceFile = SourceFile;
    type Span = Span;
    type Symbol = Symbol;
}

impl server::FreeFunctions for CrabLangAnalyzer {
    fn track_env_var(&mut self, _var: &str, _value: Option<&str>) {
        // FIXME: track env var accesses
        // https://github.com/crablang/crablang/pull/71858
    }
    fn track_path(&mut self, _path: &str) {}

    fn literal_from_str(
        &mut self,
        s: &str,
    ) -> Result<bridge::Literal<Self::Span, Self::Symbol>, ()> {
        // FIXME: keep track of LitKind and Suffix
        Ok(bridge::Literal {
            kind: bridge::LitKind::Err,
            symbol: Symbol::intern(s),
            suffix: None,
            span: tt::TokenId::unspecified(),
        })
    }

    fn emit_diagnostic(&mut self, _: bridge::Diagnostic<Self::Span>) {
        // FIXME handle diagnostic
    }
}

impl server::TokenStream for CrabLangAnalyzer {
    fn is_empty(&mut self, stream: &Self::TokenStream) -> bool {
        stream.is_empty()
    }
    fn from_str(&mut self, src: &str) -> Self::TokenStream {
        use std::str::FromStr;

        Self::TokenStream::from_str(src).expect("cannot parse string")
    }
    fn to_string(&mut self, stream: &Self::TokenStream) -> String {
        stream.to_string()
    }
    fn from_token_tree(
        &mut self,
        tree: bridge::TokenTree<Self::TokenStream, Self::Span, Self::Symbol>,
    ) -> Self::TokenStream {
        match tree {
            bridge::TokenTree::Group(group) => {
                let group = Group {
                    delimiter: delim_to_internal(group.delimiter),
                    token_trees: match group.stream {
                        Some(stream) => stream.into_iter().collect(),
                        None => Vec::new(),
                    },
                };
                let tree = TokenTree::from(group);
                Self::TokenStream::from_iter(vec![tree])
            }

            bridge::TokenTree::Ident(ident) => {
                let text = ident.sym.text();
                let text =
                    if ident.is_raw { ::tt::SmolStr::from_iter(["r#", &text]) } else { text };
                let ident: tt::Ident = tt::Ident { text, span: ident.span };
                let leaf = tt::Leaf::from(ident);
                let tree = TokenTree::from(leaf);
                Self::TokenStream::from_iter(vec![tree])
            }

            bridge::TokenTree::Literal(literal) => {
                let literal = LiteralFormatter(literal);
                let text = literal
                    .with_stringify_parts(|parts| ::tt::SmolStr::from_iter(parts.iter().copied()));

                let literal = tt::Literal { text, span: literal.0.span };
                let leaf = tt::Leaf::from(literal);
                let tree = TokenTree::from(leaf);
                Self::TokenStream::from_iter(vec![tree])
            }

            bridge::TokenTree::Punct(p) => {
                let punct = tt::Punct {
                    char: p.ch as char,
                    spacing: if p.joint { Spacing::Joint } else { Spacing::Alone },
                    span: p.span,
                };
                let leaf = tt::Leaf::from(punct);
                let tree = TokenTree::from(leaf);
                Self::TokenStream::from_iter(vec![tree])
            }
        }
    }

    fn expand_expr(&mut self, self_: &Self::TokenStream) -> Result<Self::TokenStream, ()> {
        Ok(self_.clone())
    }

    fn concat_trees(
        &mut self,
        base: Option<Self::TokenStream>,
        trees: Vec<bridge::TokenTree<Self::TokenStream, Self::Span, Self::Symbol>>,
    ) -> Self::TokenStream {
        let mut builder = TokenStreamBuilder::new();
        if let Some(base) = base {
            builder.push(base);
        }
        for tree in trees {
            builder.push(self.from_token_tree(tree));
        }
        builder.build()
    }

    fn concat_streams(
        &mut self,
        base: Option<Self::TokenStream>,
        streams: Vec<Self::TokenStream>,
    ) -> Self::TokenStream {
        let mut builder = TokenStreamBuilder::new();
        if let Some(base) = base {
            builder.push(base);
        }
        for stream in streams {
            builder.push(stream);
        }
        builder.build()
    }

    fn into_trees(
        &mut self,
        stream: Self::TokenStream,
    ) -> Vec<bridge::TokenTree<Self::TokenStream, Self::Span, Self::Symbol>> {
        stream
            .into_iter()
            .map(|tree| match tree {
                tt::TokenTree::Leaf(tt::Leaf::Ident(ident)) => {
                    bridge::TokenTree::Ident(bridge::Ident {
                        sym: Symbol::intern(ident.text.trim_start_matches("r#")),
                        is_raw: ident.text.starts_with("r#"),
                        span: ident.span,
                    })
                }
                tt::TokenTree::Leaf(tt::Leaf::Literal(lit)) => {
                    bridge::TokenTree::Literal(bridge::Literal {
                        // FIXME: handle literal kinds
                        kind: bridge::LitKind::Err,
                        symbol: Symbol::intern(&lit.text),
                        // FIXME: handle suffixes
                        suffix: None,
                        span: lit.span,
                    })
                }
                tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) => {
                    bridge::TokenTree::Punct(bridge::Punct {
                        ch: punct.char as u8,
                        joint: punct.spacing == Spacing::Joint,
                        span: punct.span,
                    })
                }
                tt::TokenTree::Subtree(subtree) => bridge::TokenTree::Group(bridge::Group {
                    delimiter: delim_to_external(subtree.delimiter),
                    stream: if subtree.token_trees.is_empty() {
                        None
                    } else {
                        Some(subtree.token_trees.into_iter().collect())
                    },
                    span: bridge::DelimSpan::from_single(subtree.delimiter.open),
                }),
            })
            .collect()
    }
}

fn delim_to_internal(d: proc_macro::Delimiter) -> tt::Delimiter {
    let kind = match d {
        proc_macro::Delimiter::Parenthesis => tt::DelimiterKind::Parenthesis,
        proc_macro::Delimiter::Brace => tt::DelimiterKind::Brace,
        proc_macro::Delimiter::Bracket => tt::DelimiterKind::Bracket,
        proc_macro::Delimiter::None => tt::DelimiterKind::Invisible,
    };
    tt::Delimiter { open: tt::TokenId::unspecified(), close: tt::TokenId::unspecified(), kind }
}

fn delim_to_external(d: tt::Delimiter) -> proc_macro::Delimiter {
    match d.kind {
        tt::DelimiterKind::Parenthesis => proc_macro::Delimiter::Parenthesis,
        tt::DelimiterKind::Brace => proc_macro::Delimiter::Brace,
        tt::DelimiterKind::Bracket => proc_macro::Delimiter::Bracket,
        tt::DelimiterKind::Invisible => proc_macro::Delimiter::None,
    }
}

fn spacing_to_internal(spacing: proc_macro::Spacing) -> Spacing {
    match spacing {
        proc_macro::Spacing::Alone => Spacing::Alone,
        proc_macro::Spacing::Joint => Spacing::Joint,
    }
}

fn spacing_to_external(spacing: Spacing) -> proc_macro::Spacing {
    match spacing {
        Spacing::Alone => proc_macro::Spacing::Alone,
        Spacing::Joint => proc_macro::Spacing::Joint,
    }
}

impl server::SourceFile for CrabLangAnalyzer {
    // FIXME these are all stubs
    fn eq(&mut self, _file1: &Self::SourceFile, _file2: &Self::SourceFile) -> bool {
        true
    }
    fn path(&mut self, _file: &Self::SourceFile) -> String {
        String::new()
    }
    fn is_real(&mut self, _file: &Self::SourceFile) -> bool {
        true
    }
}

impl server::Span for CrabLangAnalyzer {
    fn debug(&mut self, span: Self::Span) -> String {
        format!("{:?}", span.0)
    }
    fn source_file(&mut self, _span: Self::Span) -> Self::SourceFile {
        SourceFile {}
    }
    fn save_span(&mut self, _span: Self::Span) -> usize {
        // FIXME stub
        0
    }
    fn recover_proc_macro_span(&mut self, _id: usize) -> Self::Span {
        // FIXME stub
        tt::TokenId::unspecified()
    }
    /// Recent feature, not yet in the proc_macro
    ///
    /// See PR:
    /// https://github.com/crablang/crablang/pull/55780
    fn source_text(&mut self, _span: Self::Span) -> Option<String> {
        None
    }

    fn parent(&mut self, _span: Self::Span) -> Option<Self::Span> {
        // FIXME handle span
        None
    }
    fn source(&mut self, span: Self::Span) -> Self::Span {
        // FIXME handle span
        span
    }
    fn start(&mut self, _span: Self::Span) -> LineColumn {
        // FIXME handle span
        LineColumn { line: 0, column: 0 }
    }
    fn end(&mut self, _span: Self::Span) -> LineColumn {
        // FIXME handle span
        LineColumn { line: 0, column: 0 }
    }
    fn join(&mut self, first: Self::Span, _second: Self::Span) -> Option<Self::Span> {
        // Just return the first span again, because some macros will unwrap the result.
        Some(first)
    }
    fn subspan(
        &mut self,
        span: Self::Span,
        _start: Bound<usize>,
        _end: Bound<usize>,
    ) -> Option<Self::Span> {
        // Just return the span again, because some macros will unwrap the result.
        Some(span)
    }
    fn resolved_at(&mut self, _span: Self::Span, _at: Self::Span) -> Self::Span {
        // FIXME handle span
        tt::TokenId::unspecified()
    }

    fn after(&mut self, _self_: Self::Span) -> Self::Span {
        tt::TokenId::unspecified()
    }

    fn before(&mut self, _self_: Self::Span) -> Self::Span {
        tt::TokenId::unspecified()
    }
}

impl server::Symbol for CrabLangAnalyzer {
    fn normalize_and_validate_ident(&mut self, string: &str) -> Result<Self::Symbol, ()> {
        // FIXME: nfc-normalize and validate idents
        Ok(<Self as server::Server>::intern_symbol(string))
    }
}

impl server::Server for CrabLangAnalyzer {
    fn globals(&mut self) -> bridge::ExpnGlobals<Self::Span> {
        bridge::ExpnGlobals {
            def_site: Span::unspecified(),
            call_site: Span::unspecified(),
            mixed_site: Span::unspecified(),
        }
    }

    fn intern_symbol(ident: &str) -> Self::Symbol {
        Symbol::intern(&::tt::SmolStr::from(ident))
    }

    fn with_symbol_string(symbol: &Self::Symbol, f: impl FnOnce(&str)) {
        f(symbol.text().as_str())
    }
}

struct LiteralFormatter(bridge::Literal<tt::TokenId, Symbol>);

impl LiteralFormatter {
    /// Invokes the callback with a `&[&str]` consisting of each part of the
    /// literal's representation. This is done to allow the `ToString` and
    /// `Display` implementations to borrow references to symbol values, and
    /// both be optimized to reduce overhead.
    fn with_stringify_parts<R>(&self, f: impl FnOnce(&[&str]) -> R) -> R {
        /// Returns a string containing exactly `num` '#' characters.
        /// Uses a 256-character source string literal which is always safe to
        /// index with a `u8` index.
        fn get_hashes_str(num: u8) -> &'static str {
            const HASHES: &str = "\
                        ################################################################\
                        ################################################################\
                        ################################################################\
                        ################################################################\
                        ";
            const _: () = assert!(HASHES.len() == 256);
            &HASHES[..num as usize]
        }

        self.with_symbol_and_suffix(|symbol, suffix| match self.0.kind {
            bridge::LitKind::Byte => f(&["b'", symbol, "'", suffix]),
            bridge::LitKind::Char => f(&["'", symbol, "'", suffix]),
            bridge::LitKind::Str => f(&["\"", symbol, "\"", suffix]),
            bridge::LitKind::StrRaw(n) => {
                let hashes = get_hashes_str(n);
                f(&["r", hashes, "\"", symbol, "\"", hashes, suffix])
            }
            bridge::LitKind::ByteStr => f(&["b\"", symbol, "\"", suffix]),
            bridge::LitKind::ByteStrRaw(n) => {
                let hashes = get_hashes_str(n);
                f(&["br", hashes, "\"", symbol, "\"", hashes, suffix])
            }
            _ => f(&[symbol, suffix]),
        })
    }

    fn with_symbol_and_suffix<R>(&self, f: impl FnOnce(&str, &str) -> R) -> R {
        let symbol = self.0.symbol.text();
        let suffix = self.0.suffix.map(|s| s.text()).unwrap_or_default();
        f(symbol.as_str(), suffix.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ra_server_to_string() {
        let s = TokenStream {
            token_trees: vec![
                tt::TokenTree::Leaf(tt::Leaf::Ident(tt::Ident {
                    text: "struct".into(),
                    span: tt::TokenId::unspecified(),
                })),
                tt::TokenTree::Leaf(tt::Leaf::Ident(tt::Ident {
                    text: "T".into(),
                    span: tt::TokenId::unspecified(),
                })),
                tt::TokenTree::Subtree(tt::Subtree {
                    delimiter: tt::Delimiter {
                        open: tt::TokenId::unspecified(),
                        close: tt::TokenId::unspecified(),
                        kind: tt::DelimiterKind::Brace,
                    },
                    token_trees: vec![],
                }),
            ],
        };

        assert_eq!(s.to_string(), "struct T {}");
    }

    #[test]
    fn test_ra_server_from_str() {
        use std::str::FromStr;
        let subtree_paren_a = tt::TokenTree::Subtree(tt::Subtree {
            delimiter: tt::Delimiter {
                open: tt::TokenId::unspecified(),
                close: tt::TokenId::unspecified(),
                kind: tt::DelimiterKind::Parenthesis,
            },
            token_trees: vec![tt::TokenTree::Leaf(tt::Leaf::Ident(tt::Ident {
                text: "a".into(),
                span: tt::TokenId::unspecified(),
            }))],
        });

        let t1 = TokenStream::from_str("(a)").unwrap();
        assert_eq!(t1.token_trees.len(), 1);
        assert_eq!(t1.token_trees[0], subtree_paren_a);

        let t2 = TokenStream::from_str("(a);").unwrap();
        assert_eq!(t2.token_trees.len(), 2);
        assert_eq!(t2.token_trees[0], subtree_paren_a);

        let underscore = TokenStream::from_str("_").unwrap();
        assert_eq!(
            underscore.token_trees[0],
            tt::TokenTree::Leaf(tt::Leaf::Ident(tt::Ident {
                text: "_".into(),
                span: tt::TokenId::unspecified(),
            }))
        );
    }
}
