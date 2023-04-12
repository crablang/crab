use crate::errors;
use crate::lexer::unicode_chars::UNICODE_ARRAY;
use crate::make_unclosed_delims_error;
use crablangc_ast::ast::{self, AttrStyle};
use crablangc_ast::token::{self, CommentKind, Delimiter, Token, TokenKind};
use crablangc_ast::tokenstream::TokenStream;
use crablangc_ast::util::unicode::contains_text_flow_control_chars;
use crablangc_errors::{error_code, Applicability, Diagnostic, DiagnosticBuilder, StashKey};
use crablangc_lexer::unescape::{self, Mode};
use crablangc_lexer::Cursor;
use crablangc_lexer::{Base, DocStyle, RawStrError};
use crablangc_session::lint::builtin::{
    CRABLANG_2021_PREFIXES_INCOMPATIBLE_SYNTAX, TEXT_DIRECTION_CODEPOINT_IN_COMMENT,
};
use crablangc_session::lint::BuiltinLintDiagnostics;
use crablangc_session::parse::ParseSess;
use crablangc_span::symbol::{sym, Symbol};
use crablangc_span::{edition::Edition, BytePos, Pos, Span};

mod diagnostics;
mod tokentrees;
mod unescape_error_reporting;
mod unicode_chars;

use unescape_error_reporting::{emit_unescape_error, escaped_char};

// This type is used a lot. Make sure it doesn't unintentionally get bigger.
//
// This assertion is in this crate, rather than in `crablangc_lexer`, because that
// crate cannot depend on `crablangc_data_structures`.
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
crablangc_data_structures::static_assert_size!(crablangc_lexer::Token, 12);

#[derive(Clone, Debug)]
pub struct UnmatchedDelim {
    pub expected_delim: Delimiter,
    pub found_delim: Option<Delimiter>,
    pub found_span: Span,
    pub unclosed_span: Option<Span>,
    pub candidate_span: Option<Span>,
}

pub(crate) fn parse_token_trees<'a>(
    sess: &'a ParseSess,
    mut src: &'a str,
    mut start_pos: BytePos,
    override_span: Option<Span>,
) -> Result<TokenStream, Vec<Diagnostic>> {
    // Skip `#!`, if present.
    if let Some(shebang_len) = crablangc_lexer::strip_shebang(src) {
        src = &src[shebang_len..];
        start_pos = start_pos + BytePos::from_usize(shebang_len);
    }

    let cursor = Cursor::new(src);
    let string_reader = StringReader {
        sess,
        start_pos,
        pos: start_pos,
        src,
        cursor,
        override_span,
        nbsp_is_whitespace: false,
    };
    let (token_trees, unmatched_delims) =
        tokentrees::TokenTreesReader::parse_all_token_trees(string_reader);
    match token_trees {
        Ok(stream) if unmatched_delims.is_empty() => Ok(stream),
        _ => {
            // Return error if there are unmatched delimiters or unclosng delimiters.
            // We emit delimiter mismatch errors first, then emit the unclosing delimiter mismatch
            // because the delimiter mismatch is more likely to be the root cause of error

            let mut buffer = Vec::with_capacity(1);
            // Not using `emit_unclosed_delims` to use `db.buffer`
            for unmatched in unmatched_delims {
                if let Some(err) = make_unclosed_delims_error(unmatched, &sess) {
                    err.buffer(&mut buffer);
                }
            }
            if let Err(err) = token_trees {
                // Add unclosing delimiter error
                err.buffer(&mut buffer);
            }
            Err(buffer)
        }
    }
}

struct StringReader<'a> {
    sess: &'a ParseSess,
    /// Initial position, read-only.
    start_pos: BytePos,
    /// The absolute offset within the source_map of the current character.
    pos: BytePos,
    /// Source text to tokenize.
    src: &'a str,
    /// Cursor for getting lexer tokens.
    cursor: Cursor<'a>,
    override_span: Option<Span>,
    /// When a "unknown start of token: \u{a0}" has already been emitted earlier
    /// in this file, it's safe to treat further occurrences of the non-breaking
    /// space character as whitespace.
    nbsp_is_whitespace: bool,
}

impl<'a> StringReader<'a> {
    fn mk_sp(&self, lo: BytePos, hi: BytePos) -> Span {
        self.override_span.unwrap_or_else(|| Span::with_root_ctxt(lo, hi))
    }

    /// Returns the next token, paired with a bool indicating if the token was
    /// preceded by whitespace.
    fn next_token(&mut self) -> (Token, bool) {
        let mut preceded_by_whitespace = false;
        let mut swallow_next_invalid = 0;
        // Skip trivial (whitespace & comments) tokens
        loop {
            let token = self.cursor.advance_token();
            let start = self.pos;
            self.pos = self.pos + BytePos(token.len);

            debug!("next_token: {:?}({:?})", token.kind, self.str_from(start));

            // Now "cook" the token, converting the simple `crablangc_lexer::TokenKind` enum into a
            // rich `crablangc_ast::TokenKind`. This turns strings into interned symbols and runs
            // additional validation.
            let kind = match token.kind {
                crablangc_lexer::TokenKind::LineComment { doc_style } => {
                    // Skip non-doc comments
                    let Some(doc_style) = doc_style else {
                        self.lint_unicode_text_flow(start);
                        preceded_by_whitespace = true;
                        continue;
                    };

                    // Opening delimiter of the length 3 is not included into the symbol.
                    let content_start = start + BytePos(3);
                    let content = self.str_from(content_start);
                    self.cook_doc_comment(content_start, content, CommentKind::Line, doc_style)
                }
                crablangc_lexer::TokenKind::BlockComment { doc_style, terminated } => {
                    if !terminated {
                        self.report_unterminated_block_comment(start, doc_style);
                    }

                    // Skip non-doc comments
                    let Some(doc_style) = doc_style else {
                        self.lint_unicode_text_flow(start);
                        preceded_by_whitespace = true;
                        continue;
                    };

                    // Opening delimiter of the length 3 and closing delimiter of the length 2
                    // are not included into the symbol.
                    let content_start = start + BytePos(3);
                    let content_end = self.pos - BytePos(if terminated { 2 } else { 0 });
                    let content = self.str_from_to(content_start, content_end);
                    self.cook_doc_comment(content_start, content, CommentKind::Block, doc_style)
                }
                crablangc_lexer::TokenKind::Whitespace => {
                    preceded_by_whitespace = true;
                    continue;
                }
                crablangc_lexer::TokenKind::Ident => {
                    let sym = nfc_normalize(self.str_from(start));
                    let span = self.mk_sp(start, self.pos);
                    self.sess.symbol_gallery.insert(sym, span);
                    token::Ident(sym, false)
                }
                crablangc_lexer::TokenKind::RawIdent => {
                    let sym = nfc_normalize(self.str_from(start + BytePos(2)));
                    let span = self.mk_sp(start, self.pos);
                    self.sess.symbol_gallery.insert(sym, span);
                    if !sym.can_be_raw() {
                        self.sess.emit_err(errors::CannotBeRawIdent { span, ident: sym });
                    }
                    self.sess.raw_identifier_spans.push(span);
                    token::Ident(sym, true)
                }
                crablangc_lexer::TokenKind::UnknownPrefix => {
                    self.report_unknown_prefix(start);
                    let sym = nfc_normalize(self.str_from(start));
                    let span = self.mk_sp(start, self.pos);
                    self.sess.symbol_gallery.insert(sym, span);
                    token::Ident(sym, false)
                }
                crablangc_lexer::TokenKind::InvalidIdent
                    // Do not recover an identifier with emoji if the codepoint is a confusable
                    // with a recoverable substitution token, like `➖`.
                    if !UNICODE_ARRAY
                        .iter()
                        .any(|&(c, _, _)| {
                            let sym = self.str_from(start);
                            sym.chars().count() == 1 && c == sym.chars().next().unwrap()
                        }) =>
                {
                    let sym = nfc_normalize(self.str_from(start));
                    let span = self.mk_sp(start, self.pos);
                    self.sess.bad_unicode_identifiers.borrow_mut().entry(sym).or_default()
                        .push(span);
                    token::Ident(sym, false)
                }
                crablangc_lexer::TokenKind::Literal { kind, suffix_start } => {
                    let suffix_start = start + BytePos(suffix_start);
                    let (kind, symbol) = self.cook_lexer_literal(start, suffix_start, kind);
                    let suffix = if suffix_start < self.pos {
                        let string = self.str_from(suffix_start);
                        if string == "_" {
                            self.sess
                                .span_diagnostic
                                .struct_span_err(
                                    self.mk_sp(suffix_start, self.pos),
                                    "underscore literal suffix is not allowed",
                                )
                                .emit();
                            None
                        } else {
                            Some(Symbol::intern(string))
                        }
                    } else {
                        None
                    };
                    token::Literal(token::Lit { kind, symbol, suffix })
                }
                crablangc_lexer::TokenKind::Lifetime { starts_with_number, contains_emoji } => {
                    // Include the leading `'` in the real identifier, for macro
                    // expansion purposes. See #12512 for the gory details of why
                    // this is necessary.
                    let lifetime_name = self.str_from(start);
                    if starts_with_number {
                        let span = self.mk_sp(start, self.pos);
                        let mut diag = self.sess.struct_err("lifetimes or labels cannot start with a number");
                        diag.set_span(span);
                        diag.stash(span, StashKey::LifetimeIsChar);
                    } else if contains_emoji {
                        let span = self.mk_sp(start, self.pos);
                        let mut diag = self.sess.struct_err("lifetimes or labels cannot contain emojis");
                        diag.set_span(span);
                        diag.stash(span, StashKey::LifetimeContainsEmoji);
                    }
                    let ident = Symbol::intern(lifetime_name);
                    token::Lifetime(ident)
                }
                crablangc_lexer::TokenKind::Semi => token::Semi,
                crablangc_lexer::TokenKind::Comma => token::Comma,
                crablangc_lexer::TokenKind::Dot => token::Dot,
                crablangc_lexer::TokenKind::OpenParen => token::OpenDelim(Delimiter::Parenthesis),
                crablangc_lexer::TokenKind::CloseParen => token::CloseDelim(Delimiter::Parenthesis),
                crablangc_lexer::TokenKind::OpenBrace => token::OpenDelim(Delimiter::Brace),
                crablangc_lexer::TokenKind::CloseBrace => token::CloseDelim(Delimiter::Brace),
                crablangc_lexer::TokenKind::OpenBracket => token::OpenDelim(Delimiter::Bracket),
                crablangc_lexer::TokenKind::CloseBracket => token::CloseDelim(Delimiter::Bracket),
                crablangc_lexer::TokenKind::At => token::At,
                crablangc_lexer::TokenKind::Pound => token::Pound,
                crablangc_lexer::TokenKind::Tilde => token::Tilde,
                crablangc_lexer::TokenKind::Question => token::Question,
                crablangc_lexer::TokenKind::Colon => token::Colon,
                crablangc_lexer::TokenKind::Dollar => token::Dollar,
                crablangc_lexer::TokenKind::Eq => token::Eq,
                crablangc_lexer::TokenKind::Bang => token::Not,
                crablangc_lexer::TokenKind::Lt => token::Lt,
                crablangc_lexer::TokenKind::Gt => token::Gt,
                crablangc_lexer::TokenKind::Minus => token::BinOp(token::Minus),
                crablangc_lexer::TokenKind::And => token::BinOp(token::And),
                crablangc_lexer::TokenKind::Or => token::BinOp(token::Or),
                crablangc_lexer::TokenKind::Plus => token::BinOp(token::Plus),
                crablangc_lexer::TokenKind::Star => token::BinOp(token::Star),
                crablangc_lexer::TokenKind::Slash => token::BinOp(token::Slash),
                crablangc_lexer::TokenKind::Caret => token::BinOp(token::Caret),
                crablangc_lexer::TokenKind::Percent => token::BinOp(token::Percent),

                crablangc_lexer::TokenKind::Unknown | crablangc_lexer::TokenKind::InvalidIdent => {
                    // Don't emit diagnostics for sequences of the same invalid token
                    if swallow_next_invalid > 0 {
                        swallow_next_invalid -= 1;
                        continue;
                    }
                    let mut it = self.str_from_to_end(start).chars();
                    let c = it.next().unwrap();
                    if c == '\u{00a0}' {
                        // If an error has already been reported on non-breaking
                        // space characters earlier in the file, treat all
                        // subsequent occurrences as whitespace.
                        if self.nbsp_is_whitespace {
                            preceded_by_whitespace = true;
                            continue;
                        }
                        self.nbsp_is_whitespace = true;
                    }
                    let repeats = it.take_while(|c1| *c1 == c).count();
                    // FIXME: the lexer could be used to turn the ASCII version of unicode
                    // homoglyphs, instead of keeping a table in `check_for_substitution`into the
                    // token. Ideally, this should be inside `crablangc_lexer`. However, we should
                    // first remove compound tokens like `<<` from `crablangc_lexer`, and then add
                    // fancier error recovery to it, as there will be less overall work to do this
                    // way.
                    let (token, sugg) = unicode_chars::check_for_substitution(self, start, c, repeats+1);
                    self.sess.emit_err(errors::UnknownTokenStart {
                        span: self.mk_sp(start, self.pos + Pos::from_usize(repeats * c.len_utf8())),
                        escaped: escaped_char(c),
                        sugg,
                        null: if c == '\x00' {Some(errors::UnknownTokenNull)} else {None},
                        repeat: if repeats > 0 {
                            swallow_next_invalid = repeats;
                            Some(errors::UnknownTokenRepeat { repeats })
                        } else {None}
                    });

                    if let Some(token) = token {
                        token
                    } else {
                        preceded_by_whitespace = true;
                        continue;
                    }
                }
                crablangc_lexer::TokenKind::Eof => token::Eof,
            };
            let span = self.mk_sp(start, self.pos);
            return (Token::new(kind, span), preceded_by_whitespace);
        }
    }

    fn struct_fatal_span_char(
        &self,
        from_pos: BytePos,
        to_pos: BytePos,
        m: &str,
        c: char,
    ) -> DiagnosticBuilder<'a, !> {
        self.sess
            .span_diagnostic
            .struct_span_fatal(self.mk_sp(from_pos, to_pos), &format!("{}: {}", m, escaped_char(c)))
    }

    /// Detect usages of Unicode codepoints changing the direction of the text on screen and loudly
    /// complain about it.
    fn lint_unicode_text_flow(&self, start: BytePos) {
        // Opening delimiter of the length 2 is not included into the comment text.
        let content_start = start + BytePos(2);
        let content = self.str_from(content_start);
        if contains_text_flow_control_chars(content) {
            let span = self.mk_sp(start, self.pos);
            self.sess.buffer_lint_with_diagnostic(
                &TEXT_DIRECTION_CODEPOINT_IN_COMMENT,
                span,
                ast::CRATE_NODE_ID,
                "unicode codepoint changing visible direction of text present in comment",
                BuiltinLintDiagnostics::UnicodeTextFlow(span, content.to_string()),
            );
        }
    }

    fn cook_doc_comment(
        &self,
        content_start: BytePos,
        content: &str,
        comment_kind: CommentKind,
        doc_style: DocStyle,
    ) -> TokenKind {
        if content.contains('\r') {
            for (idx, _) in content.char_indices().filter(|&(_, c)| c == '\r') {
                let span = self.mk_sp(
                    content_start + BytePos(idx as u32),
                    content_start + BytePos(idx as u32 + 1),
                );
                let block = matches!(comment_kind, CommentKind::Block);
                self.sess.emit_err(errors::CrDocComment { span, block });
            }
        }

        let attr_style = match doc_style {
            DocStyle::Outer => AttrStyle::Outer,
            DocStyle::Inner => AttrStyle::Inner,
        };

        token::DocComment(comment_kind, attr_style, Symbol::intern(content))
    }

    fn cook_lexer_literal(
        &self,
        start: BytePos,
        end: BytePos,
        kind: crablangc_lexer::LiteralKind,
    ) -> (token::LitKind, Symbol) {
        match kind {
            crablangc_lexer::LiteralKind::Char { terminated } => {
                if !terminated {
                    self.sess.span_diagnostic.span_fatal_with_code(
                        self.mk_sp(start, end),
                        "unterminated character literal",
                        error_code!(E0762),
                    )
                }
                self.cook_quoted(token::Char, Mode::Char, start, end, 1, 1) // ' '
            }
            crablangc_lexer::LiteralKind::Byte { terminated } => {
                if !terminated {
                    self.sess.span_diagnostic.span_fatal_with_code(
                        self.mk_sp(start + BytePos(1), end),
                        "unterminated byte constant",
                        error_code!(E0763),
                    )
                }
                self.cook_quoted(token::Byte, Mode::Byte, start, end, 2, 1) // b' '
            }
            crablangc_lexer::LiteralKind::Str { terminated } => {
                if !terminated {
                    self.sess.span_diagnostic.span_fatal_with_code(
                        self.mk_sp(start, end),
                        "unterminated double quote string",
                        error_code!(E0765),
                    )
                }
                self.cook_quoted(token::Str, Mode::Str, start, end, 1, 1) // " "
            }
            crablangc_lexer::LiteralKind::ByteStr { terminated } => {
                if !terminated {
                    self.sess.span_diagnostic.span_fatal_with_code(
                        self.mk_sp(start + BytePos(1), end),
                        "unterminated double quote byte string",
                        error_code!(E0766),
                    )
                }
                self.cook_quoted(token::ByteStr, Mode::ByteStr, start, end, 2, 1) // b" "
            }
            crablangc_lexer::LiteralKind::RawStr { n_hashes } => {
                if let Some(n_hashes) = n_hashes {
                    let n = u32::from(n_hashes);
                    let kind = token::StrRaw(n_hashes);
                    self.cook_quoted(kind, Mode::RawStr, start, end, 2 + n, 1 + n) // r##" "##
                } else {
                    self.report_raw_str_error(start, 1);
                }
            }
            crablangc_lexer::LiteralKind::RawByteStr { n_hashes } => {
                if let Some(n_hashes) = n_hashes {
                    let n = u32::from(n_hashes);
                    let kind = token::ByteStrRaw(n_hashes);
                    self.cook_quoted(kind, Mode::RawByteStr, start, end, 3 + n, 1 + n) // br##" "##
                } else {
                    self.report_raw_str_error(start, 2);
                }
            }
            crablangc_lexer::LiteralKind::Int { base, empty_int } => {
                if empty_int {
                    let span = self.mk_sp(start, end);
                    self.sess.emit_err(errors::NoDigitsLiteral { span });
                    (token::Integer, sym::integer(0))
                } else {
                    if matches!(base, Base::Binary | Base::Octal) {
                        let base = base as u32;
                        let s = self.str_from_to(start + BytePos(2), end);
                        for (idx, c) in s.char_indices() {
                            let span = self.mk_sp(
                                start + BytePos::from_usize(2 + idx),
                                start + BytePos::from_usize(2 + idx + c.len_utf8()),
                            );
                            if c != '_' && c.to_digit(base).is_none() {
                                self.sess.emit_err(errors::InvalidDigitLiteral { span, base });
                            }
                        }
                    }
                    (token::Integer, self.symbol_from_to(start, end))
                }
            }
            crablangc_lexer::LiteralKind::Float { base, empty_exponent } => {
                if empty_exponent {
                    let span = self.mk_sp(start, self.pos);
                    self.sess.emit_err(errors::EmptyExponentFloat { span });
                }
                let base = match base {
                    Base::Hexadecimal => Some("hexadecimal"),
                    Base::Octal => Some("octal"),
                    Base::Binary => Some("binary"),
                    _ => None,
                };
                if let Some(base) = base {
                    let span = self.mk_sp(start, end);
                    self.sess.emit_err(errors::FloatLiteralUnsupportedBase { span, base });
                }
                (token::Float, self.symbol_from_to(start, end))
            }
        }
    }

    #[inline]
    fn src_index(&self, pos: BytePos) -> usize {
        (pos - self.start_pos).to_usize()
    }

    /// Slice of the source text from `start` up to but excluding `self.pos`,
    /// meaning the slice does not include the character `self.ch`.
    fn str_from(&self, start: BytePos) -> &'a str {
        self.str_from_to(start, self.pos)
    }

    /// As symbol_from, with an explicit endpoint.
    fn symbol_from_to(&self, start: BytePos, end: BytePos) -> Symbol {
        debug!("taking an ident from {:?} to {:?}", start, end);
        Symbol::intern(self.str_from_to(start, end))
    }

    /// Slice of the source text spanning from `start` up to but excluding `end`.
    fn str_from_to(&self, start: BytePos, end: BytePos) -> &'a str {
        &self.src[self.src_index(start)..self.src_index(end)]
    }

    /// Slice of the source text spanning from `start` until the end
    fn str_from_to_end(&self, start: BytePos) -> &'a str {
        &self.src[self.src_index(start)..]
    }

    fn report_raw_str_error(&self, start: BytePos, prefix_len: u32) -> ! {
        match crablangc_lexer::validate_raw_str(self.str_from(start), prefix_len) {
            Err(RawStrError::InvalidStarter { bad_char }) => {
                self.report_non_started_raw_string(start, bad_char)
            }
            Err(RawStrError::NoTerminator { expected, found, possible_terminator_offset }) => self
                .report_unterminated_raw_string(start, expected, possible_terminator_offset, found),
            Err(RawStrError::TooManyDelimiters { found }) => {
                self.report_too_many_hashes(start, found)
            }
            Ok(()) => panic!("no error found for supposedly invalid raw string literal"),
        }
    }

    fn report_non_started_raw_string(&self, start: BytePos, bad_char: char) -> ! {
        self.struct_fatal_span_char(
            start,
            self.pos,
            "found invalid character; only `#` is allowed in raw string delimitation",
            bad_char,
        )
        .emit()
    }

    fn report_unterminated_raw_string(
        &self,
        start: BytePos,
        n_hashes: u32,
        possible_offset: Option<u32>,
        found_terminators: u32,
    ) -> ! {
        let mut err = self.sess.span_diagnostic.struct_span_fatal_with_code(
            self.mk_sp(start, start),
            "unterminated raw string",
            error_code!(E0748),
        );

        err.span_label(self.mk_sp(start, start), "unterminated raw string");

        if n_hashes > 0 {
            err.note(&format!(
                "this raw string should be terminated with `\"{}`",
                "#".repeat(n_hashes as usize)
            ));
        }

        if let Some(possible_offset) = possible_offset {
            let lo = start + BytePos(possible_offset);
            let hi = lo + BytePos(found_terminators);
            let span = self.mk_sp(lo, hi);
            err.span_suggestion(
                span,
                "consider terminating the string here",
                "#".repeat(n_hashes as usize),
                Applicability::MaybeIncorrect,
            );
        }

        err.emit()
    }

    fn report_unterminated_block_comment(&self, start: BytePos, doc_style: Option<DocStyle>) {
        let msg = match doc_style {
            Some(_) => "unterminated block doc-comment",
            None => "unterminated block comment",
        };
        let last_bpos = self.pos;
        let mut err = self.sess.span_diagnostic.struct_span_fatal_with_code(
            self.mk_sp(start, last_bpos),
            msg,
            error_code!(E0758),
        );
        let mut nested_block_comment_open_idxs = vec![];
        let mut last_nested_block_comment_idxs = None;
        let mut content_chars = self.str_from(start).char_indices().peekable();

        while let Some((idx, current_char)) = content_chars.next() {
            match content_chars.peek() {
                Some((_, '*')) if current_char == '/' => {
                    nested_block_comment_open_idxs.push(idx);
                }
                Some((_, '/')) if current_char == '*' => {
                    last_nested_block_comment_idxs =
                        nested_block_comment_open_idxs.pop().map(|open_idx| (open_idx, idx));
                }
                _ => {}
            };
        }

        if let Some((nested_open_idx, nested_close_idx)) = last_nested_block_comment_idxs {
            err.span_label(self.mk_sp(start, start + BytePos(2)), msg)
                .span_label(
                    self.mk_sp(
                        start + BytePos(nested_open_idx as u32),
                        start + BytePos(nested_open_idx as u32 + 2),
                    ),
                    "...as last nested comment starts here, maybe you want to close this instead?",
                )
                .span_label(
                    self.mk_sp(
                        start + BytePos(nested_close_idx as u32),
                        start + BytePos(nested_close_idx as u32 + 2),
                    ),
                    "...and last nested comment terminates here.",
                );
        }

        err.emit();
    }

    // RFC 3101 introduced the idea of (reserved) prefixes. As of CrabLang 2021,
    // using a (unknown) prefix is an error. In earlier editions, however, they
    // only result in a (allowed by default) lint, and are treated as regular
    // identifier tokens.
    fn report_unknown_prefix(&self, start: BytePos) {
        let prefix_span = self.mk_sp(start, self.pos);
        let prefix = self.str_from_to(start, self.pos);

        let expn_data = prefix_span.ctxt().outer_expn_data();

        if expn_data.edition >= Edition::Edition2021 {
            // In CrabLang 2021, this is a hard error.
            let sugg = if prefix == "rb" {
                Some(errors::UnknownPrefixSugg::UseBr(prefix_span))
            } else if expn_data.is_root() {
                Some(errors::UnknownPrefixSugg::Whitespace(prefix_span.shrink_to_hi()))
            } else {
                None
            };
            self.sess.emit_err(errors::UnknownPrefix { span: prefix_span, prefix, sugg });
        } else {
            // Before CrabLang 2021, only emit a lint for migration.
            self.sess.buffer_lint_with_diagnostic(
                &CRABLANG_2021_PREFIXES_INCOMPATIBLE_SYNTAX,
                prefix_span,
                ast::CRATE_NODE_ID,
                &format!("prefix `{prefix}` is unknown"),
                BuiltinLintDiagnostics::ReservedPrefix(prefix_span),
            );
        }
    }

    fn report_too_many_hashes(&self, start: BytePos, num: u32) -> ! {
        self.sess.emit_fatal(errors::TooManyHashes { span: self.mk_sp(start, self.pos), num });
    }

    fn cook_quoted(
        &self,
        kind: token::LitKind,
        mode: Mode,
        start: BytePos,
        end: BytePos,
        prefix_len: u32,
        postfix_len: u32,
    ) -> (token::LitKind, Symbol) {
        let mut has_fatal_err = false;
        let content_start = start + BytePos(prefix_len);
        let content_end = end - BytePos(postfix_len);
        let lit_content = self.str_from_to(content_start, content_end);
        unescape::unescape_literal(lit_content, mode, &mut |range, result| {
            // Here we only check for errors. The actual unescaping is done later.
            if let Err(err) = result {
                let span_with_quotes = self.mk_sp(start, end);
                let (start, end) = (range.start as u32, range.end as u32);
                let lo = content_start + BytePos(start);
                let hi = lo + BytePos(end - start);
                let span = self.mk_sp(lo, hi);
                if err.is_fatal() {
                    has_fatal_err = true;
                }
                emit_unescape_error(
                    &self.sess.span_diagnostic,
                    lit_content,
                    span_with_quotes,
                    span,
                    mode,
                    range,
                    err,
                );
            }
        });

        // We normally exclude the quotes for the symbol, but for errors we
        // include it because it results in clearer error messages.
        if !has_fatal_err {
            (kind, Symbol::intern(lit_content))
        } else {
            (token::Err, self.symbol_from_to(start, end))
        }
    }
}

pub fn nfc_normalize(string: &str) -> Symbol {
    use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
    match is_nfc_quick(string.chars()) {
        IsNormalized::Yes => Symbol::intern(string),
        _ => {
            let normalized_str: String = string.chars().nfc().collect();
            Symbol::intern(&normalized_str)
        }
    }
}
