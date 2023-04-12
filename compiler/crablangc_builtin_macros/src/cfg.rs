//! The compiler code necessary to support the cfg! extension, which expands to
//! a literal `true` or `false` based on whether the given cfg matches the
//! current compilation environment.

use crate::errors;
use crablangc_ast as ast;
use crablangc_ast::token;
use crablangc_ast::tokenstream::TokenStream;
use crablangc_attr as attr;
use crablangc_errors::PResult;
use crablangc_expand::base::{self, *};
use crablangc_span::Span;

pub fn expand_cfg(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let sp = cx.with_def_site_ctxt(sp);

    match parse_cfg(cx, sp, tts) {
        Ok(cfg) => {
            let matches_cfg = attr::cfg_matches(
                &cfg,
                &cx.sess.parse_sess,
                cx.current_expansion.lint_node_id,
                cx.ecfg.features,
            );
            MacEager::expr(cx.expr_bool(sp, matches_cfg))
        }
        Err(mut err) => {
            err.emit();
            DummyResult::any(sp)
        }
    }
}

fn parse_cfg<'a>(cx: &mut ExtCtxt<'a>, span: Span, tts: TokenStream) -> PResult<'a, ast::MetaItem> {
    let mut p = cx.new_parser_from_tts(tts);

    if p.token == token::Eof {
        return Err(cx.create_err(errors::RequiresCfgPattern { span }));
    }

    let cfg = p.parse_meta_item()?;

    let _ = p.eat(&token::Comma);

    if !p.eat(&token::Eof) {
        return Err(cx.create_err(errors::OneCfgPattern { span }));
    }

    Ok(cfg)
}
