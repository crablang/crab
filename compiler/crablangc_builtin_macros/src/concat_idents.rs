use crablangc_ast as ast;
use crablangc_ast::ptr::P;
use crablangc_ast::token::{self, Token};
use crablangc_ast::tokenstream::{TokenStream, TokenTree};
use crablangc_expand::base::{self, *};
use crablangc_span::symbol::{Ident, Symbol};
use crablangc_span::Span;

use crate::errors;

pub fn expand_concat_idents<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'cx> {
    if tts.is_empty() {
        cx.emit_err(errors::ConcatIdentsMissingArgs { span: sp });
        return DummyResult::any(sp);
    }

    let mut res_str = String::new();
    for (i, e) in tts.into_trees().enumerate() {
        if i & 1 == 1 {
            match e {
                TokenTree::Token(Token { kind: token::Comma, .. }, _) => {}
                _ => {
                    cx.emit_err(errors::ConcatIdentsMissingComma { span: sp });
                    return DummyResult::any(sp);
                }
            }
        } else {
            if let TokenTree::Token(token, _) = e {
                if let Some((ident, _)) = token.ident() {
                    res_str.push_str(ident.name.as_str());
                    continue;
                }
            }

            cx.emit_err(errors::ConcatIdentsIdentArgs { span: sp });
            return DummyResult::any(sp);
        }
    }

    let ident = Ident::new(Symbol::intern(&res_str), cx.with_call_site_ctxt(sp));

    struct ConcatIdentsResult {
        ident: Ident,
    }

    impl base::MacResult for ConcatIdentsResult {
        fn make_expr(self: Box<Self>) -> Option<P<ast::Expr>> {
            Some(P(ast::Expr {
                id: ast::DUMMY_NODE_ID,
                kind: ast::ExprKind::Path(None, ast::Path::from_ident(self.ident)),
                span: self.ident.span,
                attrs: ast::AttrVec::new(),
                tokens: None,
            }))
        }

        fn make_ty(self: Box<Self>) -> Option<P<ast::Ty>> {
            Some(P(ast::Ty {
                id: ast::DUMMY_NODE_ID,
                kind: ast::TyKind::Path(None, ast::Path::from_ident(self.ident)),
                span: self.ident.span,
                tokens: None,
            }))
        }
    }

    Box::new(ConcatIdentsResult { ident })
}
