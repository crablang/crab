use crablangc_ast::tokenstream::TokenStream;
use crablangc_ast_pretty::ppcrablang;
use crablangc_expand::base;

pub fn expand_log_syntax<'cx>(
    _cx: &'cx mut base::ExtCtxt<'_>,
    sp: crablangc_span::Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'cx> {
    println!("{}", ppcrablang::tts_to_string(&tts));

    // any so that `log_syntax` can be invoked as an expression and item.
    base::DummyResult::any_valid(sp)
}
