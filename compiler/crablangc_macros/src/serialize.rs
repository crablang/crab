use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;
use syn::spanned::Spanned;

pub fn type_decodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    let decoder_ty = quote! { __D };
    if !s.ast().generics.lifetimes().any(|lt| lt.lifetime.ident == "tcx") {
        s.add_impl_generic(parse_quote! { 'tcx });
    }
    s.add_impl_generic(parse_quote! {#decoder_ty: ::crablangc_type_ir::codec::TyDecoder<I = ::crablangc_middle::ty::TyCtxt<'tcx>>});
    s.add_bounds(synstructure::AddBounds::Generics);

    decodable_body(s, decoder_ty)
}

pub fn meta_decodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    if !s.ast().generics.lifetimes().any(|lt| lt.lifetime.ident == "tcx") {
        s.add_impl_generic(parse_quote! { 'tcx });
    }
    s.add_impl_generic(parse_quote! { '__a });
    let decoder_ty = quote! { DecodeContext<'__a, 'tcx> };
    s.add_bounds(synstructure::AddBounds::Generics);

    decodable_body(s, decoder_ty)
}

pub fn decodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    let decoder_ty = quote! { __D };
    s.add_impl_generic(parse_quote! {#decoder_ty: ::crablangc_serialize::Decoder});
    s.add_bounds(synstructure::AddBounds::Generics);

    decodable_body(s, decoder_ty)
}

fn decodable_body(
    s: synstructure::Structure<'_>,
    decoder_ty: TokenStream,
) -> proc_macro2::TokenStream {
    if let syn::Data::Union(_) = s.ast().data {
        panic!("cannot derive on union")
    }
    let ty_name = s.ast().ident.to_string();
    let decode_body = match s.variants() {
        [] => {
            let message = format!("`{}` has no variants to decode", ty_name);
            quote! {
                panic!(#message)
            }
        }
        [vi] => vi.construct(|field, _index| decode_field(field)),
        variants => {
            let match_inner: TokenStream = variants
                .iter()
                .enumerate()
                .map(|(idx, vi)| {
                    let construct = vi.construct(|field, _index| decode_field(field));
                    quote! { #idx => { #construct } }
                })
                .collect();
            let message = format!(
                "invalid enum variant tag while decoding `{}`, expected 0..{}",
                ty_name,
                variants.len()
            );
            quote! {
                match ::crablangc_serialize::Decoder::read_usize(__decoder) {
                    #match_inner
                    _ => panic!(#message),
                }
            }
        }
    };

    s.bound_impl(
        quote!(::crablangc_serialize::Decodable<#decoder_ty>),
        quote! {
            fn decode(__decoder: &mut #decoder_ty) -> Self {
                #decode_body
            }
        },
    )
}

fn decode_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_span = field.ident.as_ref().map_or(field.ty.span(), |ident| ident.span());

    let decode_inner_method = if let syn::Type::Reference(_) = field.ty {
        quote! { ::crablangc_middle::ty::codec::RefDecodable::decode }
    } else {
        quote! { ::crablangc_serialize::Decodable::decode }
    };
    let __decoder = quote! { __decoder };
    // Use the span of the field for the method call, so
    // that backtraces will point to the field.
    quote_spanned! {field_span=> #decode_inner_method(#__decoder) }
}

pub fn type_encodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    if !s.ast().generics.lifetimes().any(|lt| lt.lifetime.ident == "tcx") {
        s.add_impl_generic(parse_quote! {'tcx});
    }
    let encoder_ty = quote! { __E };
    s.add_impl_generic(parse_quote! {#encoder_ty: ::crablangc_type_ir::codec::TyEncoder<I = ::crablangc_middle::ty::TyCtxt<'tcx>>});
    s.add_bounds(synstructure::AddBounds::Generics);

    encodable_body(s, encoder_ty, false)
}

pub fn meta_encodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    if !s.ast().generics.lifetimes().any(|lt| lt.lifetime.ident == "tcx") {
        s.add_impl_generic(parse_quote! {'tcx});
    }
    s.add_impl_generic(parse_quote! { '__a });
    let encoder_ty = quote! { EncodeContext<'__a, 'tcx> };
    s.add_bounds(synstructure::AddBounds::Generics);

    encodable_body(s, encoder_ty, true)
}

pub fn encodable_derive(mut s: synstructure::Structure<'_>) -> proc_macro2::TokenStream {
    let encoder_ty = quote! { __E };
    s.add_impl_generic(parse_quote! { #encoder_ty: ::crablangc_serialize::Encoder});
    s.add_bounds(synstructure::AddBounds::Generics);

    encodable_body(s, encoder_ty, false)
}

fn encodable_body(
    mut s: synstructure::Structure<'_>,
    encoder_ty: TokenStream,
    allow_unreachable_code: bool,
) -> proc_macro2::TokenStream {
    if let syn::Data::Union(_) = s.ast().data {
        panic!("cannot derive on union")
    }

    s.bind_with(|binding| {
        // Handle the lack of a blanket reference impl.
        if let syn::Type::Reference(_) = binding.ast().ty {
            synstructure::BindStyle::Move
        } else {
            synstructure::BindStyle::Ref
        }
    });

    let encode_body = match s.variants() {
        [] => {
            quote! {
                match *self {}
            }
        }
        [_] => {
            let encode_inner = s.each_variant(|vi| {
                vi.bindings()
                    .iter()
                    .map(|binding| {
                        let bind_ident = &binding.binding;
                        let result = quote! {
                            ::crablangc_serialize::Encodable::<#encoder_ty>::encode(
                                #bind_ident,
                                __encoder,
                            );
                        };
                        result
                    })
                    .collect::<TokenStream>()
            });
            quote! {
                match *self { #encode_inner }
            }
        }
        _ => {
            let disc = {
                let mut variant_idx = 0usize;
                let encode_inner = s.each_variant(|_| {
                    let result = quote! {
                        #variant_idx
                    };
                    variant_idx += 1;
                    result
                });
                quote! {
                    let disc = match *self {
                        #encode_inner
                    };
                    ::crablangc_serialize::Encoder::emit_usize(__encoder, disc);
                }
            };

            let mut variant_idx = 0usize;
            let encode_inner = s.each_variant(|vi| {
                let encode_fields: TokenStream = vi
                    .bindings()
                    .iter()
                    .map(|binding| {
                        let bind_ident = &binding.binding;
                        let result = quote! {
                            ::crablangc_serialize::Encodable::<#encoder_ty>::encode(
                                #bind_ident,
                                __encoder,
                            );
                        };
                        result
                    })
                    .collect();
                variant_idx += 1;
                encode_fields
            });
            quote! {
                #disc
                match *self {
                    #encode_inner
                }
            }
        }
    };

    let lints = if allow_unreachable_code {
        quote! { #![allow(unreachable_code)] }
    } else {
        quote! {}
    };

    s.bound_impl(
        quote!(::crablangc_serialize::Encodable<#encoder_ty>),
        quote! {
            fn encode(
                &self,
                __encoder: &mut #encoder_ty,
            ) {
                #lints
                #encode_body
            }
        },
    )
}
