//! Exports a few trivial procedural macros for testing.

#![warn(crablang_2018_idioms, unused_lifetimes, semicolon_in_expressions_from_macros)]

use proc_macro::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

#[proc_macro]
pub fn fn_like_noop(args: TokenStream) -> TokenStream {
    args
}

#[proc_macro]
pub fn fn_like_panic(args: TokenStream) -> TokenStream {
    panic!("fn_like_panic!({})", args);
}

#[proc_macro]
pub fn fn_like_error(args: TokenStream) -> TokenStream {
    format!("compile_error!(\"fn_like_error!({})\");", args).parse().unwrap()
}

#[proc_macro]
pub fn fn_like_clone_tokens(args: TokenStream) -> TokenStream {
    clone_stream(args)
}

#[proc_macro]
pub fn fn_like_mk_literals(_args: TokenStream) -> TokenStream {
    let trees: Vec<TokenTree> = vec![
        TokenTree::from(Literal::byte_string(b"byte_string")),
        TokenTree::from(Literal::character('c')),
        TokenTree::from(Literal::string("string")),
        // as of 2022-07-21, there's no method on `Literal` to build a raw
        // string or a raw byte string
        TokenTree::from(Literal::f64_suffixed(3.14)),
        TokenTree::from(Literal::f64_unsuffixed(3.14)),
        TokenTree::from(Literal::i64_suffixed(123)),
        TokenTree::from(Literal::i64_unsuffixed(123)),
    ];
    TokenStream::from_iter(trees)
}

#[proc_macro]
pub fn fn_like_mk_idents(_args: TokenStream) -> TokenStream {
    let trees: Vec<TokenTree> = vec![
        TokenTree::from(Ident::new("standard", Span::call_site())),
        TokenTree::from(Ident::new_raw("raw", Span::call_site())),
    ];
    TokenStream::from_iter(trees)
}

#[proc_macro_attribute]
pub fn attr_noop(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn attr_panic(args: TokenStream, item: TokenStream) -> TokenStream {
    panic!("#[attr_panic {}] {}", args, item);
}

#[proc_macro_attribute]
pub fn attr_error(args: TokenStream, item: TokenStream) -> TokenStream {
    format!("compile_error!(\"#[attr_error({})] {}\");", args, item).parse().unwrap()
}

#[proc_macro_derive(DeriveEmpty)]
pub fn derive_empty(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(DerivePanic)]
pub fn derive_panic(item: TokenStream) -> TokenStream {
    panic!("#[derive(DerivePanic)] {}", item);
}

#[proc_macro_derive(DeriveError)]
pub fn derive_error(item: TokenStream) -> TokenStream {
    format!("compile_error!(\"#[derive(DeriveError)] {}\");", item).parse().unwrap()
}

fn clone_stream(ts: TokenStream) -> TokenStream {
    ts.into_iter().map(clone_tree).collect()
}

fn clone_tree(t: TokenTree) -> TokenTree {
    match t {
        TokenTree::Group(orig) => {
            let mut new = Group::new(orig.delimiter(), clone_stream(orig.stream()));
            new.set_span(orig.span());
            TokenTree::Group(new)
        }
        TokenTree::Ident(orig) => {
            let s = orig.to_string();
            if let Some(rest) = s.strip_prefix("r#") {
                TokenTree::Ident(Ident::new_raw(rest, orig.span()))
            } else {
                TokenTree::Ident(Ident::new(&s, orig.span()))
            }
        }
        TokenTree::Punct(orig) => {
            let mut new = Punct::new(orig.as_char(), orig.spacing());
            new.set_span(orig.span());
            TokenTree::Punct(new)
        }
        TokenTree::Literal(orig) => {
            // this goes through `literal_from_str` as of 2022-07-18, cf.
            // https://github.com/crablang/crablang/commit/b34c79f8f1ef4d0149ad4bf77e1759c07a9a01a8
            let mut new: Literal = orig.to_string().parse().unwrap();
            new.set_span(orig.span());
            TokenTree::Literal(new)
        }
    }
}
