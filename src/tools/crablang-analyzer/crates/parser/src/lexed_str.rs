//! Lexing `&str` into a sequence of CrabLang tokens.
//!
//! Note that strictly speaking the parser in this crate is not required to work
//! on tokens which originated from text. Macros, eg, can synthesize tokens out
//! of thin air. So, ideally, lexer should be an orthogonal crate. It is however
//! convenient to include a text-based lexer here!
//!
//! Note that these tokens, unlike the tokens we feed into the parser, do
//! include info about comments and whitespace.

use std::ops;

use crate::{
    SyntaxKind::{self, *},
    T,
};

pub struct LexedStr<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
}

struct LexError {
    msg: String,
    token: u32,
}

impl<'a> LexedStr<'a> {
    pub fn new(text: &'a str) -> LexedStr<'a> {
        let mut conv = Converter::new(text);
        if let Some(shebang_len) = crablangc_lexer::strip_shebang(text) {
            conv.res.push(SHEBANG, conv.offset);
            conv.offset = shebang_len;
        };

        for token in crablangc_lexer::tokenize(&text[conv.offset..]) {
            let token_text = &text[conv.offset..][..token.len];

            conv.extend_token(&token.kind, token_text);
        }

        conv.finalize_with_eof()
    }

    pub fn single_token(text: &'a str) -> Option<(SyntaxKind, Option<String>)> {
        if text.is_empty() {
            return None;
        }

        let token = crablangc_lexer::first_token(text);
        if token.len != text.len() {
            return None;
        }

        let mut conv = Converter::new(text);
        conv.extend_token(&token.kind, text);
        match &*conv.res.kind {
            [kind] => Some((*kind, conv.res.error.pop().map(|it| it.msg))),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        self.text
    }

    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn kind(&self, i: usize) -> SyntaxKind {
        assert!(i < self.len());
        self.kind[i]
    }

    pub fn text(&self, i: usize) -> &str {
        self.range_text(i..i + 1)
    }

    pub fn range_text(&self, r: ops::Range<usize>) -> &str {
        assert!(r.start < r.end && r.end <= self.len());
        let lo = self.start[r.start] as usize;
        let hi = self.start[r.end] as usize;
        &self.text[lo..hi]
    }

    // Naming is hard.
    pub fn text_range(&self, i: usize) -> ops::Range<usize> {
        assert!(i < self.len());
        let lo = self.start[i] as usize;
        let hi = self.start[i + 1] as usize;
        lo..hi
    }
    pub fn text_start(&self, i: usize) -> usize {
        assert!(i <= self.len());
        self.start[i] as usize
    }
    pub fn text_len(&self, i: usize) -> usize {
        assert!(i < self.len());
        let r = self.text_range(i);
        r.end - r.start
    }

    pub fn error(&self, i: usize) -> Option<&str> {
        assert!(i < self.len());
        let err = self.error.binary_search_by_key(&(i as u32), |i| i.token).ok()?;
        Some(self.error[err].msg.as_str())
    }

    pub fn errors(&self) -> impl Iterator<Item = (usize, &str)> + '_ {
        self.error.iter().map(|it| (it.token as usize, it.msg.as_str()))
    }

    fn push(&mut self, kind: SyntaxKind, offset: usize) {
        self.kind.push(kind);
        self.start.push(offset as u32);
    }
}

struct Converter<'a> {
    res: LexedStr<'a>,
    offset: usize,
}

impl<'a> Converter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            res: LexedStr { text, kind: Vec::new(), start: Vec::new(), error: Vec::new() },
            offset: 0,
        }
    }

    fn finalize_with_eof(mut self) -> LexedStr<'a> {
        self.res.push(EOF, self.offset);
        self.res
    }

    fn push(&mut self, kind: SyntaxKind, len: usize, err: Option<&str>) {
        self.res.push(kind, self.offset);
        self.offset += len;

        if let Some(err) = err {
            let token = self.res.len() as u32;
            let msg = err.to_string();
            self.res.error.push(LexError { msg, token });
        }
    }

    fn extend_token(&mut self, kind: &crablangc_lexer::TokenKind, token_text: &str) {
        // A note on an intended tradeoff:
        // We drop some useful information here (see patterns with double dots `..`)
        // Storing that info in `SyntaxKind` is not possible due to its layout requirements of
        // being `u16` that come from `rowan::SyntaxKind`.
        let mut err = "";

        let syntax_kind = {
            match kind {
                crablangc_lexer::TokenKind::LineComment { doc_style: _ } => COMMENT,
                crablangc_lexer::TokenKind::BlockComment { doc_style: _, terminated } => {
                    if !terminated {
                        err = "Missing trailing `*/` symbols to terminate the block comment";
                    }
                    COMMENT
                }

                crablangc_lexer::TokenKind::Whitespace => WHITESPACE,

                crablangc_lexer::TokenKind::Ident if token_text == "_" => UNDERSCORE,
                crablangc_lexer::TokenKind::Ident => {
                    SyntaxKind::from_keyword(token_text).unwrap_or(IDENT)
                }

                crablangc_lexer::TokenKind::RawIdent => IDENT,
                crablangc_lexer::TokenKind::Literal { kind, .. } => {
                    self.extend_literal(token_text.len(), kind);
                    return;
                }

                crablangc_lexer::TokenKind::Lifetime { starts_with_number } => {
                    if *starts_with_number {
                        err = "Lifetime name cannot start with a number";
                    }
                    LIFETIME_IDENT
                }

                crablangc_lexer::TokenKind::Semi => T![;],
                crablangc_lexer::TokenKind::Comma => T![,],
                crablangc_lexer::TokenKind::Dot => T![.],
                crablangc_lexer::TokenKind::OpenParen => T!['('],
                crablangc_lexer::TokenKind::CloseParen => T![')'],
                crablangc_lexer::TokenKind::OpenBrace => T!['{'],
                crablangc_lexer::TokenKind::CloseBrace => T!['}'],
                crablangc_lexer::TokenKind::OpenBracket => T!['['],
                crablangc_lexer::TokenKind::CloseBracket => T![']'],
                crablangc_lexer::TokenKind::At => T![@],
                crablangc_lexer::TokenKind::Pound => T![#],
                crablangc_lexer::TokenKind::Tilde => T![~],
                crablangc_lexer::TokenKind::Question => T![?],
                crablangc_lexer::TokenKind::Colon => T![:],
                crablangc_lexer::TokenKind::Dollar => T![$],
                crablangc_lexer::TokenKind::Eq => T![=],
                crablangc_lexer::TokenKind::Bang => T![!],
                crablangc_lexer::TokenKind::Lt => T![<],
                crablangc_lexer::TokenKind::Gt => T![>],
                crablangc_lexer::TokenKind::Minus => T![-],
                crablangc_lexer::TokenKind::And => T![&],
                crablangc_lexer::TokenKind::Or => T![|],
                crablangc_lexer::TokenKind::Plus => T![+],
                crablangc_lexer::TokenKind::Star => T![*],
                crablangc_lexer::TokenKind::Slash => T![/],
                crablangc_lexer::TokenKind::Caret => T![^],
                crablangc_lexer::TokenKind::Percent => T![%],
                crablangc_lexer::TokenKind::Unknown => ERROR,
                crablangc_lexer::TokenKind::UnknownPrefix => {
                    err = "unknown literal prefix";
                    IDENT
                }
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, token_text.len(), err);
    }

    fn extend_literal(&mut self, len: usize, kind: &crablangc_lexer::LiteralKind) {
        let mut err = "";

        let syntax_kind = match *kind {
            crablangc_lexer::LiteralKind::Int { empty_int, base: _ } => {
                if empty_int {
                    err = "Missing digits after the integer base prefix";
                }
                INT_NUMBER
            }
            crablangc_lexer::LiteralKind::Float { empty_exponent, base: _ } => {
                if empty_exponent {
                    err = "Missing digits after the exponent symbol";
                }
                FLOAT_NUMBER
            }
            crablangc_lexer::LiteralKind::Char { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the character literal";
                }
                CHAR
            }
            crablangc_lexer::LiteralKind::Byte { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the byte literal";
                }
                BYTE
            }
            crablangc_lexer::LiteralKind::Str { terminated } => {
                if !terminated {
                    err = "Missing trailing `\"` symbol to terminate the string literal";
                }
                STRING
            }
            crablangc_lexer::LiteralKind::ByteStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `\"` symbol to terminate the byte string literal";
                }
                BYTE_STRING
            }
            crablangc_lexer::LiteralKind::RawStr { err: raw_str_err, .. } => {
                if let Some(raw_str_err) = raw_str_err {
                    err = match raw_str_err {
                        crablangc_lexer::RawStrError::InvalidStarter { .. } => "Missing `\"` symbol after `#` symbols to begin the raw string literal",
                        crablangc_lexer::RawStrError::NoTerminator { expected, found, .. } => if expected == found {
                            "Missing trailing `\"` to terminate the raw string literal"
                        } else {
                            "Missing trailing `\"` with `#` symbols to terminate the raw string literal"
                        },
                        crablangc_lexer::RawStrError::TooManyDelimiters { .. } => "Too many `#` symbols: raw strings may be delimited by up to 65535 `#` symbols",
                    };
                };
                STRING
            }
            crablangc_lexer::LiteralKind::RawByteStr { err: raw_str_err, .. } => {
                if let Some(raw_str_err) = raw_str_err {
                    err = match raw_str_err {
                        crablangc_lexer::RawStrError::InvalidStarter { .. } => "Missing `\"` symbol after `#` symbols to begin the raw byte string literal",
                        crablangc_lexer::RawStrError::NoTerminator { expected, found, .. } => if expected == found {
                            "Missing trailing `\"` to terminate the raw byte string literal"
                        } else {
                            "Missing trailing `\"` with `#` symbols to terminate the raw byte string literal"
                        },
                        crablangc_lexer::RawStrError::TooManyDelimiters { .. } => "Too many `#` symbols: raw byte strings may be delimited by up to 65535 `#` symbols",
                    };
                };

                BYTE_STRING
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, len, err);
    }
}
