// The compiler code necessary to support the env! extension. Eventually this
// should all get sucked into either the compiler syntax extension plugin
// interface.
//

use crablangc_ast::tokenstream::TokenStream;
use crablangc_ast::{self as ast, GenericArg};
use crablangc_expand::base::{self, *};
use crablangc_span::symbol::{kw, sym, Ident, Symbol};
use crablangc_span::Span;
use std::env;
use thin_vec::thin_vec;

use crate::errors;

pub fn expand_option_env<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'cx> {
    let Some(var) = get_single_str_from_tts(cx, sp, tts, "option_env!") else {
        return DummyResult::any(sp);
    };

    let sp = cx.with_def_site_ctxt(sp);
    let value = env::var(var.as_str()).ok().as_deref().map(Symbol::intern);
    cx.sess.parse_sess.env_depinfo.borrow_mut().insert((var, value));
    let e = match value {
        None => {
            let lt = cx.lifetime(sp, Ident::new(kw::StaticLifetime, sp));
            cx.expr_path(cx.path_all(
                sp,
                true,
                cx.std_path(&[sym::option, sym::Option, sym::None]),
                vec![GenericArg::Type(cx.ty_ref(
                    sp,
                    cx.ty_ident(sp, Ident::new(sym::str, sp)),
                    Some(lt),
                    ast::Mutability::Not,
                ))],
            ))
        }
        Some(value) => cx.expr_call_global(
            sp,
            cx.std_path(&[sym::option, sym::Option, sym::Some]),
            thin_vec![cx.expr_str(sp, value)],
        ),
    };
    MacEager::expr(e)
}

pub fn expand_env<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'cx> {
    let mut exprs = match get_exprs_from_tts(cx, tts) {
        Some(exprs) if exprs.is_empty() || exprs.len() > 2 => {
            cx.emit_err(errors::EnvTakesArgs { span: sp });
            return DummyResult::any(sp);
        }
        None => return DummyResult::any(sp),
        Some(exprs) => exprs.into_iter(),
    };

    let Some((var, _style)) = expr_to_string(cx, exprs.next().unwrap(), "expected string literal") else {
        return DummyResult::any(sp);
    };

    let custom_msg = match exprs.next() {
        None => None,
        Some(second) => match expr_to_string(cx, second, "expected string literal") {
            None => return DummyResult::any(sp),
            Some((s, _style)) => Some(s),
        },
    };

    let sp = cx.with_def_site_ctxt(sp);
    let value = env::var(var.as_str()).ok().as_deref().map(Symbol::intern);
    cx.sess.parse_sess.env_depinfo.borrow_mut().insert((var, value));
    let e = match value {
        None => {
            cx.emit_err(errors::EnvNotDefined {
                span: sp,
                msg: custom_msg,
                var,
                help: custom_msg.is_none().then(|| help_for_missing_env_var(var.as_str())),
            });
            return DummyResult::any(sp);
        }
        Some(value) => cx.expr_str(sp, value),
    };
    MacEager::expr(e)
}

fn help_for_missing_env_var(var: &str) -> errors::EnvNotDefinedHelp {
    if var.starts_with("CARGO_")
        || var.starts_with("DEP_")
        || matches!(var, "OUT_DIR" | "OPT_LEVEL" | "PROFILE" | "HOST" | "TARGET")
    {
        errors::EnvNotDefinedHelp::CargoVar
    } else {
        errors::EnvNotDefinedHelp::Other
    }
}
