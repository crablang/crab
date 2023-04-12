use crablangc_ast::ast;
use crablangc_ast::ptr::P;
use crablangc_ast::token::TokenKind;
use crablangc_ast::tokenstream::TokenStream;
use crablangc_span::symbol::{self, kw};

use crate::rewrite::RewriteContext;

pub(crate) fn parse_lazy_static(
    context: &RewriteContext<'_>,
    ts: TokenStream,
) -> Option<Vec<(ast::Visibility, symbol::Ident, P<ast::Ty>, P<ast::Expr>)>> {
    let mut result = vec![];
    let mut parser = super::build_parser(context, ts);
    macro_rules! parse_or {
        ($method:ident $(,)* $($arg:expr),* $(,)*) => {
            match parser.$method($($arg,)*) {
                Ok(val) => {
                    if parser.sess.span_diagnostic.has_errors().is_some() {
                        parser.sess.span_diagnostic.reset_err_count();
                        return None;
                    } else {
                        val
                    }
                }
                Err(err) => {
                    err.cancel();
                    parser.sess.span_diagnostic.reset_err_count();
                    return None;
                }
            }
        }
    }

    while parser.token.kind != TokenKind::Eof {
        // Parse a `lazy_static!` item.
        let vis = parse_or!(parse_visibility, crablangc_parse::parser::FollowedByType::No);
        parser.eat_keyword(kw::Static);
        parser.eat_keyword(kw::Ref);
        let id = parse_or!(parse_ident);
        parser.eat(&TokenKind::Colon);
        let ty = parse_or!(parse_ty);
        parser.eat(&TokenKind::Eq);
        let expr = parse_or!(parse_expr);
        parser.eat(&TokenKind::Semi);
        result.push((vis, id, ty, expr));
    }

    Some(result)
}
