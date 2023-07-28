//! Builtin macro

use std::mem;

use ::tt::Ident;
use base_db::{AnchoredPath, Edition, FileId};
use cfg::CfgExpr;
use either::Either;
use mbe::{parse_exprs_with_sep, parse_to_token_tree, TokenMap};
use rustc_hash::FxHashMap;
use syntax::{
    ast::{self, AstToken},
    SmolStr,
};

use crate::{
    db::ExpandDatabase, name, quote, tt, EagerCallInfo, ExpandError, ExpandResult, MacroCallId,
    MacroCallLoc,
};

macro_rules! register_builtin {
    ( LAZY: $(($name:ident, $kind: ident) => $expand:ident),* , EAGER: $(($e_name:ident, $e_kind: ident) => $e_expand:ident),*  ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum BuiltinFnLikeExpander {
            $($kind),*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum EagerExpander {
            $($e_kind),*
        }

        impl BuiltinFnLikeExpander {
            pub fn expand(
                &self,
                db: &dyn ExpandDatabase,
                id: MacroCallId,
                tt: &tt::Subtree,
            ) -> ExpandResult<tt::Subtree> {
                let expander = match *self {
                    $( BuiltinFnLikeExpander::$kind => $expand, )*
                };
                expander(db, id, tt)
            }
        }

        impl EagerExpander {
            pub fn expand(
                &self,
                db: &dyn ExpandDatabase,
                arg_id: MacroCallId,
                tt: &tt::Subtree,
            ) -> ExpandResult<tt::Subtree> {
                let expander = match *self {
                    $( EagerExpander::$e_kind => $e_expand, )*
                };
                expander(db, arg_id, tt)
            }
        }

        fn find_by_name(ident: &name::Name) -> Option<Either<BuiltinFnLikeExpander, EagerExpander>> {
            match ident {
                $( id if id == &name::name![$name] => Some(Either::Left(BuiltinFnLikeExpander::$kind)), )*
                $( id if id == &name::name![$e_name] => Some(Either::Right(EagerExpander::$e_kind)), )*
                _ => return None,
            }
        }
    };
}

impl EagerExpander {
    pub fn is_include(&self) -> bool {
        matches!(self, EagerExpander::Include)
    }
}

pub fn find_builtin_macro(
    ident: &name::Name,
) -> Option<Either<BuiltinFnLikeExpander, EagerExpander>> {
    find_by_name(ident)
}

register_builtin! {
    LAZY:
    (column, Column) => column_expand,
    (file, File) => file_expand,
    (line, Line) => line_expand,
    (module_path, ModulePath) => module_path_expand,
    (assert, Assert) => assert_expand,
    (stringify, Stringify) => stringify_expand,
    (llvm_asm, LlvmAsm) => asm_expand,
    (asm, Asm) => asm_expand,
    (global_asm, GlobalAsm) => global_asm_expand,
    (cfg, Cfg) => cfg_expand,
    (core_panic, CorePanic) => panic_expand,
    (std_panic, StdPanic) => panic_expand,
    (unreachable, Unreachable) => unreachable_expand,
    (log_syntax, LogSyntax) => log_syntax_expand,
    (trace_macros, TraceMacros) => trace_macros_expand,

    EAGER:
    (format_args, FormatArgs) => format_args_expand,
    (const_format_args, ConstFormatArgs) => format_args_expand,
    (format_args_nl, FormatArgsNl) => format_args_nl_expand,
    (compile_error, CompileError) => compile_error_expand,
    (concat, Concat) => concat_expand,
    (concat_idents, ConcatIdents) => concat_idents_expand,
    (concat_bytes, ConcatBytes) => concat_bytes_expand,
    (include, Include) => include_expand,
    (include_bytes, IncludeBytes) => include_bytes_expand,
    (include_str, IncludeStr) => include_str_expand,
    (env, Env) => env_expand,
    (option_env, OptionEnv) => option_env_expand
}

const DOLLAR_CRATE: tt::Ident =
    tt::Ident { text: SmolStr::new_inline("$crate"), span: tt::TokenId::unspecified() };

fn module_path_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // Just return a dummy result.
    ExpandResult::ok(quote! { "module::path" })
}

fn line_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // dummy implementation for type-checking purposes
    let expanded = quote! {
        0 as u32
    };

    ExpandResult::ok(expanded)
}

fn log_syntax_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    ExpandResult::ok(quote! {})
}

fn trace_macros_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    ExpandResult::ok(quote! {})
}

fn stringify_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let pretty = ::tt::pretty(&tt.token_trees);

    let expanded = quote! {
        #pretty
    };

    ExpandResult::ok(expanded)
}

fn column_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // dummy implementation for type-checking purposes
    let expanded = quote! {
        0 as u32
    };

    ExpandResult::ok(expanded)
}

fn assert_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let args = parse_exprs_with_sep(tt, ',');
    let expanded = match &*args {
        [cond, panic_args @ ..] => {
            let comma = tt::Subtree {
                delimiter: tt::Delimiter::unspecified(),
                token_trees: vec![tt::TokenTree::Leaf(tt::Leaf::Punct(tt::Punct {
                    char: ',',
                    spacing: tt::Spacing::Alone,
                    span: tt::TokenId::unspecified(),
                }))],
            };
            let cond = cond.clone();
            let panic_args = itertools::Itertools::intersperse(panic_args.iter().cloned(), comma);
            quote! {{
                if !(#cond) {
                    #DOLLAR_CRATE::panic!(##panic_args);
                }
            }}
        }
        [] => quote! {{}},
    };

    ExpandResult::ok(expanded)
}

fn file_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // FIXME: RA purposefully lacks knowledge of absolute file names
    // so just return "".
    let file_name = "";

    let expanded = quote! {
        #file_name
    };

    ExpandResult::ok(expanded)
}

fn format_args_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    format_args_expand_general(db, id, tt, "")
}

fn format_args_nl_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    format_args_expand_general(db, id, tt, "\\n")
}

fn format_args_expand_general(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    tt: &tt::Subtree,
    end_string: &str,
) -> ExpandResult<tt::Subtree> {
    let args = parse_exprs_with_sep(tt, ',');

    let expand_error =
        ExpandResult::new(tt::Subtree::empty(), mbe::ExpandError::NoMatchingRule.into());

    let mut key_args = FxHashMap::default();
    let mut args = args.into_iter().filter_map(|mut arg| {
        // Remove `key =`.
        if matches!(arg.token_trees.get(1), Some(tt::TokenTree::Leaf(tt::Leaf::Punct(p))) if p.char == '=')
        {
            // but not with `==`
            if !matches!(arg.token_trees.get(2), Some(tt::TokenTree::Leaf(tt::Leaf::Punct(p))) if p.char == '=')
            {
                let key = arg.token_trees.drain(..2).next().unwrap();
                key_args.insert(key.to_string(), arg);
                return None;
            }
        }
        Some(arg)
    }).collect::<Vec<_>>().into_iter();
    // ^^^^^^^ we need this collect, to enforce the side effect of the filter_map closure (building the `key_args`)
    let Some(format_subtree) = args.next() else {
        return expand_error;
    };
    let format_string = (|| {
        let token_tree = format_subtree.token_trees.get(0)?;
        match token_tree {
            tt::TokenTree::Leaf(l) => match l {
                tt::Leaf::Literal(l) => {
                    if let Some(mut text) = l.text.strip_prefix('r') {
                        let mut raw_sharps = String::new();
                        while let Some(t) = text.strip_prefix('#') {
                            text = t;
                            raw_sharps.push('#');
                        }
                        text =
                            text.strip_suffix(&raw_sharps)?.strip_prefix('"')?.strip_suffix('"')?;
                        Some((text, l.span, Some(raw_sharps)))
                    } else {
                        let text = l.text.strip_prefix('"')?.strip_suffix('"')?;
                        let span = l.span;
                        Some((text, span, None))
                    }
                }
                _ => None,
            },
            tt::TokenTree::Subtree(_) => None,
        }
    })();
    let Some((format_string, _format_string_span, raw_sharps)) = format_string else {
        return expand_error;
    };
    let mut format_iter = format_string.chars().peekable();
    let mut parts = vec![];
    let mut last_part = String::new();
    let mut arg_tts = vec![];
    let mut err = None;
    while let Some(c) = format_iter.next() {
        // Parsing the format string. See https://doc.rust-lang.org/std/fmt/index.html#syntax for the grammar and more info
        match c {
            '{' => {
                if format_iter.peek() == Some(&'{') {
                    format_iter.next();
                    last_part.push('{');
                    continue;
                }
                let mut argument = String::new();
                while ![Some(&'}'), Some(&':')].contains(&format_iter.peek()) {
                    argument.push(match format_iter.next() {
                        Some(c) => c,
                        None => return expand_error,
                    });
                }
                let format_spec = match format_iter.next().unwrap() {
                    '}' => "".to_owned(),
                    ':' => {
                        let mut s = String::new();
                        while let Some(c) = format_iter.next() {
                            if c == '}' {
                                break;
                            }
                            s.push(c);
                        }
                        s
                    }
                    _ => unreachable!(),
                };
                parts.push(mem::take(&mut last_part));
                let arg_tree = if argument.is_empty() {
                    match args.next() {
                        Some(it) => it,
                        None => {
                            err = Some(mbe::ExpandError::NoMatchingRule.into());
                            tt::Subtree::empty()
                        }
                    }
                } else if let Some(tree) = key_args.get(&argument) {
                    tree.clone()
                } else {
                    // FIXME: we should pick the related substring of the `_format_string_span` as the span. You
                    // can use `.char_indices()` instead of `.char()` for `format_iter` to find the substring interval.
                    let ident = Ident::new(argument, tt::TokenId::unspecified());
                    quote!(#ident)
                };
                let formatter = match &*format_spec {
                    "?" => quote!(::core::fmt::Debug::fmt),
                    "" => quote!(::core::fmt::Display::fmt),
                    _ => {
                        // FIXME: implement the rest and return expand error here
                        quote!(::core::fmt::Display::fmt)
                    }
                };
                arg_tts.push(quote! { ::core::fmt::ArgumentV1::new(&(#arg_tree), #formatter), });
            }
            '}' => {
                if format_iter.peek() == Some(&'}') {
                    format_iter.next();
                    last_part.push('}');
                } else {
                    return expand_error;
                }
            }
            _ => last_part.push(c),
        }
    }
    last_part += end_string;
    if !last_part.is_empty() {
        parts.push(last_part);
    }
    let part_tts = parts.into_iter().map(|it| {
        let text = if let Some(raw) = &raw_sharps {
            format!("r{raw}\"{}\"{raw}", it).into()
        } else {
            format!("\"{}\"", it).into()
        };
        let l = tt::Literal { span: tt::TokenId::unspecified(), text };
        quote!(#l ,)
    });
    let arg_tts = arg_tts.into_iter().flat_map(|arg| arg.token_trees);
    let expanded = quote! {
        ::core::fmt::Arguments::new_v1(&[##part_tts], &[##arg_tts])
    };
    ExpandResult { value: expanded, err }
}

fn asm_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // We expand all assembly snippets to `format_args!` invocations to get format syntax
    // highlighting for them.

    let mut literals = Vec::new();
    for tt in tt.token_trees.chunks(2) {
        match tt {
            [tt::TokenTree::Leaf(tt::Leaf::Literal(lit))]
            | [tt::TokenTree::Leaf(tt::Leaf::Literal(lit)), tt::TokenTree::Leaf(tt::Leaf::Punct(tt::Punct { char: ',', span: _, spacing: _ }))] =>
            {
                let krate = DOLLAR_CRATE.clone();
                literals.push(quote!(#krate::format_args!(#lit);));
            }
            _ => break,
        }
    }

    let expanded = quote! {{
        ##literals
        loop {}
    }};
    ExpandResult::ok(expanded)
}

fn global_asm_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    // Expand to nothing (at item-level)
    ExpandResult::ok(quote! {})
}

fn cfg_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let loc = db.lookup_intern_macro_call(id);
    let expr = CfgExpr::parse(tt);
    let enabled = db.crate_graph()[loc.krate].cfg_options.check(&expr) != Some(false);
    let expanded = if enabled { quote!(true) } else { quote!(false) };
    ExpandResult::ok(expanded)
}

fn panic_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let loc: MacroCallLoc = db.lookup_intern_macro_call(id);
    // Expand to a macro call `$crate::panic::panic_{edition}`
    let mut call = if db.crate_graph()[loc.krate].edition >= Edition::Edition2021 {
        quote!(#DOLLAR_CRATE::panic::panic_2021!)
    } else {
        quote!(#DOLLAR_CRATE::panic::panic_2015!)
    };

    // Pass the original arguments
    call.token_trees.push(tt::TokenTree::Subtree(tt.clone()));
    ExpandResult::ok(call)
}

fn unreachable_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let loc: MacroCallLoc = db.lookup_intern_macro_call(id);
    // Expand to a macro call `$crate::panic::unreachable_{edition}`
    let mut call = if db.crate_graph()[loc.krate].edition >= Edition::Edition2021 {
        quote!(#DOLLAR_CRATE::panic::unreachable_2021!)
    } else {
        quote!(#DOLLAR_CRATE::panic::unreachable_2015!)
    };

    // Pass the original arguments
    call.token_trees.push(tt::TokenTree::Subtree(tt.clone()));
    ExpandResult::ok(call)
}

fn unquote_str(lit: &tt::Literal) -> Option<String> {
    let lit = ast::make::tokens::literal(&lit.to_string());
    let token = ast::String::cast(lit)?;
    token.value().map(|it| it.into_owned())
}

fn unquote_char(lit: &tt::Literal) -> Option<char> {
    let lit = ast::make::tokens::literal(&lit.to_string());
    let token = ast::Char::cast(lit)?;
    token.value()
}

fn unquote_byte_string(lit: &tt::Literal) -> Option<Vec<u8>> {
    let lit = ast::make::tokens::literal(&lit.to_string());
    let token = ast::ByteString::cast(lit)?;
    token.value().map(|it| it.into_owned())
}

fn compile_error_expand(
    _db: &dyn ExpandDatabase,
    _id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let err = match &*tt.token_trees {
        [tt::TokenTree::Leaf(tt::Leaf::Literal(it))] => match unquote_str(it) {
            Some(unquoted) => ExpandError::other(unquoted),
            None => ExpandError::other("`compile_error!` argument must be a string"),
        },
        _ => ExpandError::other("`compile_error!` argument must be a string"),
    };

    ExpandResult { value: quote! {}, err: Some(err) }
}

fn concat_expand(
    _db: &dyn ExpandDatabase,
    _arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let mut err = None;
    let mut text = String::new();
    for (i, mut t) in tt.token_trees.iter().enumerate() {
        // FIXME: hack on top of a hack: `$e:expr` captures get surrounded in parentheses
        // to ensure the right parsing order, so skip the parentheses here. Ideally we'd
        // implement rustc's model. cc https://github.com/rust-lang/rust-analyzer/pull/10623
        if let tt::TokenTree::Subtree(tt::Subtree { delimiter: delim, token_trees }) = t {
            if let [tt] = &**token_trees {
                if delim.kind == tt::DelimiterKind::Parenthesis {
                    t = tt;
                }
            }
        }

        match t {
            tt::TokenTree::Leaf(tt::Leaf::Literal(it)) if i % 2 == 0 => {
                // concat works with string and char literals, so remove any quotes.
                // It also works with integer, float and boolean literals, so just use the rest
                // as-is.
                if let Some(c) = unquote_char(it) {
                    text.push(c);
                } else {
                    let component = unquote_str(it).unwrap_or_else(|| it.text.to_string());
                    text.push_str(&component);
                }
            }
            // handle boolean literals
            tt::TokenTree::Leaf(tt::Leaf::Ident(id))
                if i % 2 == 0 && (id.text == "true" || id.text == "false") =>
            {
                text.push_str(id.text.as_str());
            }
            tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) if i % 2 == 1 && punct.char == ',' => (),
            _ => {
                err.get_or_insert(mbe::ExpandError::UnexpectedToken.into());
            }
        }
    }
    ExpandResult { value: quote!(#text), err }
}

fn concat_bytes_expand(
    _db: &dyn ExpandDatabase,
    _arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let mut bytes = Vec::new();
    let mut err = None;
    for (i, t) in tt.token_trees.iter().enumerate() {
        match t {
            tt::TokenTree::Leaf(tt::Leaf::Literal(lit)) => {
                let token = ast::make::tokens::literal(&lit.to_string());
                match token.kind() {
                    syntax::SyntaxKind::BYTE => bytes.push(token.text().to_string()),
                    syntax::SyntaxKind::BYTE_STRING => {
                        let components = unquote_byte_string(lit).unwrap_or_default();
                        components.into_iter().for_each(|it| bytes.push(it.to_string()));
                    }
                    _ => {
                        err.get_or_insert(mbe::ExpandError::UnexpectedToken.into());
                        break;
                    }
                }
            }
            tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) if i % 2 == 1 && punct.char == ',' => (),
            tt::TokenTree::Subtree(tree) if tree.delimiter.kind == tt::DelimiterKind::Bracket => {
                if let Err(e) = concat_bytes_expand_subtree(tree, &mut bytes) {
                    err.get_or_insert(e);
                    break;
                }
            }
            _ => {
                err.get_or_insert(mbe::ExpandError::UnexpectedToken.into());
                break;
            }
        }
    }
    let ident = tt::Ident { text: bytes.join(", ").into(), span: tt::TokenId::unspecified() };
    ExpandResult { value: quote!([#ident]), err }
}

fn concat_bytes_expand_subtree(
    tree: &tt::Subtree,
    bytes: &mut Vec<String>,
) -> Result<(), ExpandError> {
    for (ti, tt) in tree.token_trees.iter().enumerate() {
        match tt {
            tt::TokenTree::Leaf(tt::Leaf::Literal(lit)) => {
                let lit = ast::make::tokens::literal(&lit.to_string());
                match lit.kind() {
                    syntax::SyntaxKind::BYTE | syntax::SyntaxKind::INT_NUMBER => {
                        bytes.push(lit.text().to_string())
                    }
                    _ => {
                        return Err(mbe::ExpandError::UnexpectedToken.into());
                    }
                }
            }
            tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) if ti % 2 == 1 && punct.char == ',' => (),
            _ => {
                return Err(mbe::ExpandError::UnexpectedToken.into());
            }
        }
    }
    Ok(())
}

fn concat_idents_expand(
    _db: &dyn ExpandDatabase,
    _arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let mut err = None;
    let mut ident = String::new();
    for (i, t) in tt.token_trees.iter().enumerate() {
        match t {
            tt::TokenTree::Leaf(tt::Leaf::Ident(id)) => {
                ident.push_str(id.text.as_str());
            }
            tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) if i % 2 == 1 && punct.char == ',' => (),
            _ => {
                err.get_or_insert(mbe::ExpandError::UnexpectedToken.into());
            }
        }
    }
    let ident = tt::Ident { text: ident.into(), span: tt::TokenId::unspecified() };
    ExpandResult { value: quote!(#ident), err }
}

fn relative_file(
    db: &dyn ExpandDatabase,
    call_id: MacroCallId,
    path_str: &str,
    allow_recursion: bool,
) -> Result<FileId, ExpandError> {
    let call_site = call_id.as_file().original_file(db);
    let path = AnchoredPath { anchor: call_site, path: path_str };
    let res = db
        .resolve_path(path)
        .ok_or_else(|| ExpandError::other(format!("failed to load file `{path_str}`")))?;
    // Prevent include itself
    if res == call_site && !allow_recursion {
        Err(ExpandError::other(format!("recursive inclusion of `{path_str}`")))
    } else {
        Ok(res)
    }
}

fn parse_string(tt: &tt::Subtree) -> Result<String, ExpandError> {
    tt.token_trees
        .get(0)
        .and_then(|tt| match tt {
            tt::TokenTree::Leaf(tt::Leaf::Literal(it)) => unquote_str(it),
            _ => None,
        })
        .ok_or(mbe::ExpandError::ConversionError.into())
}

fn include_expand(
    db: &dyn ExpandDatabase,
    arg_id: MacroCallId,
    _tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    match db.include_expand(arg_id) {
        Ok((res, _)) => ExpandResult::ok(res.0.clone()),
        Err(e) => ExpandResult::new(tt::Subtree::empty(), e),
    }
}

pub(crate) fn include_arg_to_tt(
    db: &dyn ExpandDatabase,
    arg_id: MacroCallId,
) -> Result<(triomphe::Arc<(::tt::Subtree<::tt::TokenId>, TokenMap)>, FileId), ExpandError> {
    let loc = db.lookup_intern_macro_call(arg_id);
    let Some(EagerCallInfo { arg,arg_id, .. }) = loc.eager.as_deref() else {
        panic!("include_arg_to_tt called on non include macro call: {:?}", &loc.eager);
    };
    let path = parse_string(&arg.0)?;
    let file_id = relative_file(db, *arg_id, &path, false)?;

    let (subtree, map) =
        parse_to_token_tree(&db.file_text(file_id)).ok_or(mbe::ExpandError::ConversionError)?;
    Ok((triomphe::Arc::new((subtree, map)), file_id))
}

fn include_bytes_expand(
    _db: &dyn ExpandDatabase,
    _arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    if let Err(e) = parse_string(tt) {
        return ExpandResult::new(tt::Subtree::empty(), e);
    }

    // FIXME: actually read the file here if the user asked for macro expansion
    let res = tt::Subtree {
        delimiter: tt::Delimiter::unspecified(),
        token_trees: vec![tt::TokenTree::Leaf(tt::Leaf::Literal(tt::Literal {
            text: r#"b"""#.into(),
            span: tt::TokenId::unspecified(),
        }))],
    };
    ExpandResult::ok(res)
}

fn include_str_expand(
    db: &dyn ExpandDatabase,
    arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let path = match parse_string(tt) {
        Ok(it) => it,
        Err(e) => return ExpandResult::new(tt::Subtree::empty(), e),
    };

    // FIXME: we're not able to read excluded files (which is most of them because
    // it's unusual to `include_str!` a Rust file), but we can return an empty string.
    // Ideally, we'd be able to offer a precise expansion if the user asks for macro
    // expansion.
    let file_id = match relative_file(db, arg_id, &path, true) {
        Ok(file_id) => file_id,
        Err(_) => {
            return ExpandResult::ok(quote!(""));
        }
    };

    let text = db.file_text(file_id);
    let text = &*text;

    ExpandResult::ok(quote!(#text))
}

fn get_env_inner(db: &dyn ExpandDatabase, arg_id: MacroCallId, key: &str) -> Option<String> {
    let krate = db.lookup_intern_macro_call(arg_id).krate;
    db.crate_graph()[krate].env.get(key)
}

fn env_expand(
    db: &dyn ExpandDatabase,
    arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let key = match parse_string(tt) {
        Ok(it) => it,
        Err(e) => return ExpandResult::new(tt::Subtree::empty(), e),
    };

    let mut err = None;
    let s = get_env_inner(db, arg_id, &key).unwrap_or_else(|| {
        // The only variable rust-analyzer ever sets is `OUT_DIR`, so only diagnose that to avoid
        // unnecessary diagnostics for eg. `CARGO_PKG_NAME`.
        if key == "OUT_DIR" {
            err = Some(ExpandError::other(r#"`OUT_DIR` not set, enable "build scripts" to fix"#));
        }

        // If the variable is unset, still return a dummy string to help type inference along.
        // We cannot use an empty string here, because for
        // `include!(concat!(env!("OUT_DIR"), "/foo.rs"))` will become
        // `include!("foo.rs"), which might go to infinite loop
        "UNRESOLVED_ENV_VAR".to_string()
    });
    let expanded = quote! { #s };

    ExpandResult { value: expanded, err }
}

fn option_env_expand(
    db: &dyn ExpandDatabase,
    arg_id: MacroCallId,
    tt: &tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let key = match parse_string(tt) {
        Ok(it) => it,
        Err(e) => return ExpandResult::new(tt::Subtree::empty(), e),
    };
    // FIXME: Use `DOLLAR_CRATE` when that works in eager macros.
    let expanded = match get_env_inner(db, arg_id, &key) {
        None => quote! { ::core::option::Option::None::<&str> },
        Some(s) => quote! { ::core::option::Option::Some(#s) },
    };

    ExpandResult::ok(expanded)
}
