use crablangc_ast as ast;
use crablangc_ast::ptr::P;
use crablangc_ast::token;
use crablangc_ast::tokenstream::TokenStream;
use crablangc_ast_pretty::ppcrablang;
use crablangc_expand::base::{self, *};
use crablangc_expand::module::DirOwnership;
use crablangc_parse::parser::{ForceCollect, Parser};
use crablangc_parse::{self, new_parser_from_file};
use crablangc_session::lint::builtin::INCOMPLETE_INCLUDE;
use crablangc_span::symbol::Symbol;
use crablangc_span::{self, Pos, Span};

use smallvec::SmallVec;
use std::rc::Rc;

// These macros all relate to the file system; they either return
// the column/row/filename of the expression, or they include
// a given file into the current one.

/// line!(): expands to the current line number
pub fn expand_line(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    base::check_zero_tts(cx, sp, tts, "line!");

    let topmost = cx.expansion_cause().unwrap_or(sp);
    let loc = cx.source_map().lookup_char_pos(topmost.lo());

    base::MacEager::expr(cx.expr_u32(topmost, loc.line as u32))
}

/* column!(): expands to the current column number */
pub fn expand_column(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    base::check_zero_tts(cx, sp, tts, "column!");

    let topmost = cx.expansion_cause().unwrap_or(sp);
    let loc = cx.source_map().lookup_char_pos(topmost.lo());

    base::MacEager::expr(cx.expr_u32(topmost, loc.col.to_usize() as u32 + 1))
}

/// file!(): expands to the current filename */
/// The source_file (`loc.file`) contains a bunch more information we could spit
/// out if we wanted.
pub fn expand_file(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    base::check_zero_tts(cx, sp, tts, "file!");

    let topmost = cx.expansion_cause().unwrap_or(sp);
    let loc = cx.source_map().lookup_char_pos(topmost.lo());
    base::MacEager::expr(
        cx.expr_str(topmost, Symbol::intern(&loc.file.name.prefer_remapped().to_string_lossy())),
    )
}

pub fn expand_stringify(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    let s = ppcrablang::tts_to_string(&tts);
    base::MacEager::expr(cx.expr_str(sp, Symbol::intern(&s)))
}

pub fn expand_mod(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    base::check_zero_tts(cx, sp, tts, "module_path!");
    let mod_path = &cx.current_expansion.module.mod_path;
    let string = mod_path.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("::");

    base::MacEager::expr(cx.expr_str(sp, Symbol::intern(&string)))
}

/// include! : parse the given file as an expr
/// This is generally a bad idea because it's going to behave
/// unhygienically.
pub fn expand_include<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'cx> {
    let sp = cx.with_def_site_ctxt(sp);
    let Some(file) = get_single_str_from_tts(cx, sp, tts, "include!") else {
        return DummyResult::any(sp);
    };
    // The file will be added to the code map by the parser
    let file = match resolve_path(&cx.sess.parse_sess, file.as_str(), sp) {
        Ok(f) => f,
        Err(mut err) => {
            err.emit();
            return DummyResult::any(sp);
        }
    };
    let p = new_parser_from_file(cx.parse_sess(), &file, Some(sp));

    // If in the included file we have e.g., `mod bar;`,
    // then the path of `bar.rs` should be relative to the directory of `file`.
    // See https://github.com/crablang/crablang/pull/69838/files#r395217057 for a discussion.
    // `MacroExpander::fully_expand_fragment` later restores, so "stack discipline" is maintained.
    let dir_path = file.parent().unwrap_or(&file).to_owned();
    cx.current_expansion.module = Rc::new(cx.current_expansion.module.with_dir_path(dir_path));
    cx.current_expansion.dir_ownership = DirOwnership::Owned { relative: None };

    struct ExpandResult<'a> {
        p: Parser<'a>,
        node_id: ast::NodeId,
    }
    impl<'a> base::MacResult for ExpandResult<'a> {
        fn make_expr(mut self: Box<ExpandResult<'a>>) -> Option<P<ast::Expr>> {
            let r = base::parse_expr(&mut self.p)?;
            if self.p.token != token::Eof {
                self.p.sess.buffer_lint(
                    &INCOMPLETE_INCLUDE,
                    self.p.token.span,
                    self.node_id,
                    "include macro expected single expression in source",
                );
            }
            Some(r)
        }

        fn make_items(mut self: Box<ExpandResult<'a>>) -> Option<SmallVec<[P<ast::Item>; 1]>> {
            let mut ret = SmallVec::new();
            loop {
                match self.p.parse_item(ForceCollect::No) {
                    Err(mut err) => {
                        err.emit();
                        break;
                    }
                    Ok(Some(item)) => ret.push(item),
                    Ok(None) => {
                        if self.p.token != token::Eof {
                            let token = ppcrablang::token_to_string(&self.p.token);
                            let msg = format!("expected item, found `{}`", token);
                            self.p.struct_span_err(self.p.token.span, &msg).emit();
                        }

                        break;
                    }
                }
            }
            Some(ret)
        }
    }

    Box::new(ExpandResult { p, node_id: cx.current_expansion.lint_node_id })
}

/// `include_str!`: read the given file, insert it as a literal string expr
pub fn expand_include_str(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    let Some(file) = get_single_str_from_tts(cx, sp, tts, "include_str!") else {
        return DummyResult::any(sp);
    };
    let file = match resolve_path(&cx.sess.parse_sess, file.as_str(), sp) {
        Ok(f) => f,
        Err(mut err) => {
            err.emit();
            return DummyResult::any(sp);
        }
    };
    match cx.source_map().load_binary_file(&file) {
        Ok(bytes) => match std::str::from_utf8(&bytes) {
            Ok(src) => {
                let interned_src = Symbol::intern(&src);
                base::MacEager::expr(cx.expr_str(sp, interned_src))
            }
            Err(_) => {
                cx.span_err(sp, &format!("{} wasn't a utf-8 file", file.display()));
                DummyResult::any(sp)
            }
        },
        Err(e) => {
            cx.span_err(sp, &format!("couldn't read {}: {}", file.display(), e));
            DummyResult::any(sp)
        }
    }
}

pub fn expand_include_bytes(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);
    let Some(file) = get_single_str_from_tts(cx, sp, tts, "include_bytes!") else {
        return DummyResult::any(sp);
    };
    let file = match resolve_path(&cx.sess.parse_sess, file.as_str(), sp) {
        Ok(f) => f,
        Err(mut err) => {
            err.emit();
            return DummyResult::any(sp);
        }
    };
    match cx.source_map().load_binary_file(&file) {
        Ok(bytes) => {
            let expr = cx.expr(sp, ast::ExprKind::IncludedBytes(bytes.into()));
            base::MacEager::expr(expr)
        }
        Err(e) => {
            cx.span_err(sp, &format!("couldn't read {}: {}", file.display(), e));
            DummyResult::any(sp)
        }
    }
}
