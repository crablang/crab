//! Builtin derives.

use ::tt::Ident;
use base_db::{CrateOrigin, LangCrateOrigin};
use itertools::izip;
use mbe::TokenMap;
use rustc_hash::FxHashSet;
use stdx::never;
use tracing::debug;

use crate::{
    name::{AsName, Name},
    tt::{self, TokenId},
};
use syntax::ast::{self, AstNode, FieldList, HasAttrs, HasGenericParams, HasName, HasTypeBounds};

use crate::{db::ExpandDatabase, name, quote, ExpandError, ExpandResult, MacroCallId};

macro_rules! register_builtin {
    ( $($trait:ident => $expand:ident),* ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum BuiltinDeriveExpander {
            $($trait),*
        }

        impl BuiltinDeriveExpander {
            pub fn expand(
                &self,
                db: &dyn ExpandDatabase,
                id: MacroCallId,
                tt: &ast::Adt,
                token_map: &TokenMap,
            ) -> ExpandResult<tt::Subtree> {
                let expander = match *self {
                    $( BuiltinDeriveExpander::$trait => $expand, )*
                };
                expander(db, id, tt, token_map)
            }

            fn find_by_name(name: &name::Name) -> Option<Self> {
                match name {
                    $( id if id == &name::name![$trait] => Some(BuiltinDeriveExpander::$trait), )*
                     _ => None,
                }
            }
        }

    };
}

register_builtin! {
    Copy => copy_expand,
    Clone => clone_expand,
    Default => default_expand,
    Debug => debug_expand,
    Hash => hash_expand,
    Ord => ord_expand,
    PartialOrd => partial_ord_expand,
    Eq => eq_expand,
    PartialEq => partial_eq_expand
}

pub fn find_builtin_derive(ident: &name::Name) -> Option<BuiltinDeriveExpander> {
    BuiltinDeriveExpander::find_by_name(ident)
}

enum VariantShape {
    Struct(Vec<tt::Ident>),
    Tuple(usize),
    Unit,
}

fn tuple_field_iterator(n: usize) -> impl Iterator<Item = tt::Ident> {
    (0..n).map(|it| Ident::new(format!("f{it}"), tt::TokenId::unspecified()))
}

impl VariantShape {
    fn as_pattern(&self, path: tt::Subtree) -> tt::Subtree {
        self.as_pattern_map(path, |it| quote!(#it))
    }

    fn field_names(&self) -> Vec<tt::Ident> {
        match self {
            VariantShape::Struct(s) => s.clone(),
            VariantShape::Tuple(n) => tuple_field_iterator(*n).collect(),
            VariantShape::Unit => vec![],
        }
    }

    fn as_pattern_map(
        &self,
        path: tt::Subtree,
        field_map: impl Fn(&tt::Ident) -> tt::Subtree,
    ) -> tt::Subtree {
        match self {
            VariantShape::Struct(fields) => {
                let fields = fields.iter().map(|it| {
                    let mapped = field_map(it);
                    quote! { #it : #mapped , }
                });
                quote! {
                    #path { ##fields }
                }
            }
            &VariantShape::Tuple(n) => {
                let fields = tuple_field_iterator(n).map(|it| {
                    let mapped = field_map(&it);
                    quote! {
                        #mapped ,
                    }
                });
                quote! {
                    #path ( ##fields )
                }
            }
            VariantShape::Unit => path,
        }
    }

    fn from(tm: &TokenMap, value: Option<FieldList>) -> Result<Self, ExpandError> {
        let r = match value {
            None => VariantShape::Unit,
            Some(FieldList::RecordFieldList(it)) => VariantShape::Struct(
                it.fields()
                    .map(|it| it.name())
                    .map(|it| name_to_token(tm, it))
                    .collect::<Result<_, _>>()?,
            ),
            Some(FieldList::TupleFieldList(it)) => VariantShape::Tuple(it.fields().count()),
        };
        Ok(r)
    }
}

enum AdtShape {
    Struct(VariantShape),
    Enum { variants: Vec<(tt::Ident, VariantShape)>, default_variant: Option<usize> },
    Union,
}

impl AdtShape {
    fn as_pattern(&self, name: &tt::Ident) -> Vec<tt::Subtree> {
        self.as_pattern_map(name, |it| quote!(#it))
    }

    fn field_names(&self) -> Vec<Vec<tt::Ident>> {
        match self {
            AdtShape::Struct(s) => {
                vec![s.field_names()]
            }
            AdtShape::Enum { variants, .. } => {
                variants.iter().map(|(_, fields)| fields.field_names()).collect()
            }
            AdtShape::Union => {
                never!("using fields of union in derive is always wrong");
                vec![]
            }
        }
    }

    fn as_pattern_map(
        &self,
        name: &tt::Ident,
        field_map: impl Fn(&tt::Ident) -> tt::Subtree,
    ) -> Vec<tt::Subtree> {
        match self {
            AdtShape::Struct(s) => {
                vec![s.as_pattern_map(quote! { #name }, field_map)]
            }
            AdtShape::Enum { variants, .. } => variants
                .iter()
                .map(|(v, fields)| fields.as_pattern_map(quote! { #name :: #v }, &field_map))
                .collect(),
            AdtShape::Union => {
                never!("pattern matching on union is always wrong");
                vec![quote! { un }]
            }
        }
    }
}

struct BasicAdtInfo {
    name: tt::Ident,
    shape: AdtShape,
    /// first field is the name, and
    /// second field is `Some(ty)` if it's a const param of type `ty`, `None` if it's a type param.
    /// third fields is where bounds, if any
    param_types: Vec<(tt::Subtree, Option<tt::Subtree>, Option<tt::Subtree>)>,
    associated_types: Vec<tt::Subtree>,
}

fn parse_adt(tm: &TokenMap, adt: &ast::Adt) -> Result<BasicAdtInfo, ExpandError> {
    let (name, generic_param_list, shape) = match &adt {
        ast::Adt::Struct(it) => (
            it.name(),
            it.generic_param_list(),
            AdtShape::Struct(VariantShape::from(tm, it.field_list())?),
        ),
        ast::Adt::Enum(it) => {
            let default_variant = it
                .variant_list()
                .into_iter()
                .flat_map(|it| it.variants())
                .position(|it| it.attrs().any(|it| it.simple_name() == Some("default".into())));
            (
                it.name(),
                it.generic_param_list(),
                AdtShape::Enum {
                    default_variant,
                    variants: it
                        .variant_list()
                        .into_iter()
                        .flat_map(|it| it.variants())
                        .map(|it| {
                            Ok((
                                name_to_token(tm, it.name())?,
                                VariantShape::from(tm, it.field_list())?,
                            ))
                        })
                        .collect::<Result<_, ExpandError>>()?,
                },
            )
        }
        ast::Adt::Union(it) => (it.name(), it.generic_param_list(), AdtShape::Union),
    };

    let mut param_type_set: FxHashSet<Name> = FxHashSet::default();
    let param_types = generic_param_list
        .into_iter()
        .flat_map(|param_list| param_list.type_or_const_params())
        .map(|param| {
            let name = {
                let this = param.name();
                match this {
                    Some(it) => {
                        param_type_set.insert(it.as_name());
                        mbe::syntax_node_to_token_tree(it.syntax()).0
                    }
                    None => tt::Subtree::empty(),
                }
            };
            let bounds = match &param {
                ast::TypeOrConstParam::Type(it) => {
                    it.type_bound_list().map(|it| mbe::syntax_node_to_token_tree(it.syntax()).0)
                }
                ast::TypeOrConstParam::Const(_) => None,
            };
            let ty = if let ast::TypeOrConstParam::Const(param) = param {
                let ty = param
                    .ty()
                    .map(|ty| mbe::syntax_node_to_token_tree(ty.syntax()).0)
                    .unwrap_or_else(tt::Subtree::empty);
                Some(ty)
            } else {
                None
            };
            (name, ty, bounds)
        })
        .collect();

    // For a generic parameter `T`, when shorthand associated type `T::Assoc` appears in field
    // types (of any variant for enums), we generate trait bound for it. It sounds reasonable to
    // also generate trait bound for qualified associated type `<T as Trait>::Assoc`, but rustc
    // does not do that for some unknown reason.
    //
    // See the analogous function in rustc [find_type_parameters()] and rust-lang/rust#50730.
    // [find_type_parameters()]: https://github.com/rust-lang/rust/blob/1.70.0/compiler/rustc_builtin_macros/src/deriving/generic/mod.rs#L378

    // It's cumbersome to deal with the distinct structures of ADTs, so let's just get untyped
    // `SyntaxNode` that contains fields and look for descendant `ast::PathType`s. Of note is that
    // we should not inspect `ast::PathType`s in parameter bounds and where clauses.
    let field_list = match adt {
        ast::Adt::Enum(it) => it.variant_list().map(|list| list.syntax().clone()),
        ast::Adt::Struct(it) => it.field_list().map(|list| list.syntax().clone()),
        ast::Adt::Union(it) => it.record_field_list().map(|list| list.syntax().clone()),
    };
    let associated_types = field_list
        .into_iter()
        .flat_map(|it| it.descendants())
        .filter_map(ast::PathType::cast)
        .filter_map(|p| {
            let name = p.path()?.qualifier()?.as_single_name_ref()?.as_name();
            param_type_set.contains(&name).then_some(p)
        })
        .map(|it| mbe::syntax_node_to_token_tree(it.syntax()).0)
        .collect();
    let name_token = name_to_token(&tm, name)?;
    Ok(BasicAdtInfo { name: name_token, shape, param_types, associated_types })
}

fn name_to_token(token_map: &TokenMap, name: Option<ast::Name>) -> Result<tt::Ident, ExpandError> {
    let name = name.ok_or_else(|| {
        debug!("parsed item has no name");
        ExpandError::other("missing name")
    })?;
    let name_token_id =
        token_map.token_by_range(name.syntax().text_range()).unwrap_or_else(TokenId::unspecified);
    let name_token = tt::Ident { span: name_token_id, text: name.text().into() };
    Ok(name_token)
}

/// Given that we are deriving a trait `DerivedTrait` for a type like:
///
/// ```ignore (only-for-syntax-highlight)
/// struct Struct<'a, ..., 'z, A, B: DeclaredTrait, C, ..., Z> where C: WhereTrait {
///     a: A,
///     b: B::Item,
///     b1: <B as DeclaredTrait>::Item,
///     c1: <C as WhereTrait>::Item,
///     c2: Option<<C as WhereTrait>::Item>,
///     ...
/// }
/// ```
///
/// create an impl like:
///
/// ```ignore (only-for-syntax-highlight)
/// impl<'a, ..., 'z, A, B: DeclaredTrait, C, ... Z> where
///     C:                       WhereTrait,
///     A: DerivedTrait + B1 + ... + BN,
///     B: DerivedTrait + B1 + ... + BN,
///     C: DerivedTrait + B1 + ... + BN,
///     B::Item:                 DerivedTrait + B1 + ... + BN,
///     <C as WhereTrait>::Item: DerivedTrait + B1 + ... + BN,
///     ...
/// {
///     ...
/// }
/// ```
///
/// where B1, ..., BN are the bounds given by `bounds_paths`. Z is a phantom type, and
/// therefore does not get bound by the derived trait.
fn expand_simple_derive(
    tt: &ast::Adt,
    tm: &TokenMap,
    trait_path: tt::Subtree,
    make_trait_body: impl FnOnce(&BasicAdtInfo) -> tt::Subtree,
) -> ExpandResult<tt::Subtree> {
    let info = match parse_adt(tm, tt) {
        Ok(info) => info,
        Err(e) => return ExpandResult::new(tt::Subtree::empty(), e),
    };
    let trait_body = make_trait_body(&info);
    let mut where_block = vec![];
    let (params, args): (Vec<_>, Vec<_>) = info
        .param_types
        .into_iter()
        .map(|(ident, param_ty, bound)| {
            let ident_ = ident.clone();
            if let Some(b) = bound {
                let ident = ident.clone();
                where_block.push(quote! { #ident : #b , });
            }
            if let Some(ty) = param_ty {
                (quote! { const #ident : #ty , }, quote! { #ident_ , })
            } else {
                let bound = trait_path.clone();
                (quote! { #ident : #bound , }, quote! { #ident_ , })
            }
        })
        .unzip();

    where_block.extend(info.associated_types.iter().map(|it| {
        let it = it.clone();
        let bound = trait_path.clone();
        quote! { #it : #bound , }
    }));

    let name = info.name;
    let expanded = quote! {
        impl < ##params > #trait_path for #name < ##args > where ##where_block { #trait_body }
    };
    ExpandResult::ok(expanded)
}

fn find_builtin_crate(db: &dyn ExpandDatabase, id: MacroCallId) -> tt::TokenTree {
    // FIXME: make hygiene works for builtin derive macro
    // such that $crate can be used here.
    let cg = db.crate_graph();
    let krate = db.lookup_intern_macro_call(id).krate;

    let tt = if matches!(cg[krate].origin, CrateOrigin::Lang(LangCrateOrigin::Core)) {
        cov_mark::hit!(test_copy_expand_in_core);
        quote! { crate }
    } else {
        quote! { core }
    };

    tt.token_trees[0].clone()
}

fn copy_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::marker::Copy }, |_| quote! {})
}

fn clone_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::clone::Clone }, |adt| {
        if matches!(adt.shape, AdtShape::Union) {
            let star = tt::Punct {
                char: '*',
                spacing: ::tt::Spacing::Alone,
                span: tt::TokenId::unspecified(),
            };
            return quote! {
                fn clone(&self) -> Self {
                    #star self
                }
            };
        }
        if matches!(&adt.shape, AdtShape::Enum { variants, .. } if variants.is_empty()) {
            let star = tt::Punct {
                char: '*',
                spacing: ::tt::Spacing::Alone,
                span: tt::TokenId::unspecified(),
            };
            return quote! {
                fn clone(&self) -> Self {
                    match #star self {}
                }
            };
        }
        let name = &adt.name;
        let patterns = adt.shape.as_pattern(name);
        let exprs = adt.shape.as_pattern_map(name, |it| quote! { #it .clone() });
        let arms = patterns.into_iter().zip(exprs.into_iter()).map(|(pat, expr)| {
            let fat_arrow = fat_arrow();
            quote! {
                #pat #fat_arrow #expr,
            }
        });

        quote! {
            fn clone(&self) -> Self {
                match self {
                    ##arms
                }
            }
        }
    })
}

/// This function exists since `quote! { => }` doesn't work.
fn fat_arrow() -> ::tt::Subtree<TokenId> {
    let eq =
        tt::Punct { char: '=', spacing: ::tt::Spacing::Joint, span: tt::TokenId::unspecified() };
    quote! { #eq> }
}

/// This function exists since `quote! { && }` doesn't work.
fn and_and() -> ::tt::Subtree<TokenId> {
    let and =
        tt::Punct { char: '&', spacing: ::tt::Spacing::Joint, span: tt::TokenId::unspecified() };
    quote! { #and& }
}

fn default_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = &find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::default::Default }, |adt| {
        let body = match &adt.shape {
            AdtShape::Struct(fields) => {
                let name = &adt.name;
                fields
                    .as_pattern_map(quote!(#name), |_| quote!(#krate::default::Default::default()))
            }
            AdtShape::Enum { default_variant, variants } => {
                if let Some(d) = default_variant {
                    let (name, fields) = &variants[*d];
                    let adt_name = &adt.name;
                    fields.as_pattern_map(
                        quote!(#adt_name :: #name),
                        |_| quote!(#krate::default::Default::default()),
                    )
                } else {
                    // FIXME: Return expand error here
                    quote!()
                }
            }
            AdtShape::Union => {
                // FIXME: Return expand error here
                quote!()
            }
        };
        quote! {
            fn default() -> Self {
                #body
            }
        }
    })
}

fn debug_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = &find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::fmt::Debug }, |adt| {
        let for_variant = |name: String, v: &VariantShape| match v {
            VariantShape::Struct(fields) => {
                let for_fields = fields.iter().map(|it| {
                    let x_string = it.to_string();
                    quote! {
                        .field(#x_string, & #it)
                    }
                });
                quote! {
                    f.debug_struct(#name) ##for_fields .finish()
                }
            }
            VariantShape::Tuple(n) => {
                let for_fields = tuple_field_iterator(*n).map(|it| {
                    quote! {
                        .field( & #it)
                    }
                });
                quote! {
                    f.debug_tuple(#name) ##for_fields .finish()
                }
            }
            VariantShape::Unit => quote! {
                f.write_str(#name)
            },
        };
        if matches!(&adt.shape, AdtShape::Enum { variants, .. } if variants.is_empty()) {
            let star = tt::Punct {
                char: '*',
                spacing: ::tt::Spacing::Alone,
                span: tt::TokenId::unspecified(),
            };
            return quote! {
                fn fmt(&self, f: &mut #krate::fmt::Formatter) -> #krate::fmt::Result {
                    match #star self {}
                }
            };
        }
        let arms = match &adt.shape {
            AdtShape::Struct(fields) => {
                let fat_arrow = fat_arrow();
                let name = &adt.name;
                let pat = fields.as_pattern(quote!(#name));
                let expr = for_variant(name.to_string(), fields);
                vec![quote! { #pat #fat_arrow #expr }]
            }
            AdtShape::Enum { variants, .. } => variants
                .iter()
                .map(|(name, v)| {
                    let fat_arrow = fat_arrow();
                    let adt_name = &adt.name;
                    let pat = v.as_pattern(quote!(#adt_name :: #name));
                    let expr = for_variant(name.to_string(), v);
                    quote! {
                        #pat #fat_arrow #expr ,
                    }
                })
                .collect(),
            AdtShape::Union => {
                // FIXME: Return expand error here
                vec![]
            }
        };
        quote! {
            fn fmt(&self, f: &mut #krate::fmt::Formatter) -> #krate::fmt::Result {
                match self {
                    ##arms
                }
            }
        }
    })
}

fn hash_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = &find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::hash::Hash }, |adt| {
        if matches!(adt.shape, AdtShape::Union) {
            // FIXME: Return expand error here
            return quote! {};
        }
        if matches!(&adt.shape, AdtShape::Enum { variants, .. } if variants.is_empty()) {
            let star = tt::Punct {
                char: '*',
                spacing: ::tt::Spacing::Alone,
                span: tt::TokenId::unspecified(),
            };
            return quote! {
                fn hash<H: #krate::hash::Hasher>(&self, ra_expand_state: &mut H) {
                    match #star self {}
                }
            };
        }
        let arms = adt.shape.as_pattern(&adt.name).into_iter().zip(adt.shape.field_names()).map(
            |(pat, names)| {
                let expr = {
                    let it = names.iter().map(|it| quote! { #it . hash(ra_expand_state); });
                    quote! { {
                        ##it
                    } }
                };
                let fat_arrow = fat_arrow();
                quote! {
                    #pat #fat_arrow #expr ,
                }
            },
        );
        let check_discriminant = if matches!(&adt.shape, AdtShape::Enum { .. }) {
            quote! { #krate::mem::discriminant(self).hash(ra_expand_state); }
        } else {
            quote! {}
        };
        quote! {
            fn hash<H: #krate::hash::Hasher>(&self, ra_expand_state: &mut H) {
                #check_discriminant
                match self {
                    ##arms
                }
            }
        }
    })
}

fn eq_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::cmp::Eq }, |_| quote! {})
}

fn partial_eq_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::cmp::PartialEq }, |adt| {
        if matches!(adt.shape, AdtShape::Union) {
            // FIXME: Return expand error here
            return quote! {};
        }
        let name = &adt.name;

        let (self_patterns, other_patterns) = self_and_other_patterns(adt, name);
        let arms = izip!(self_patterns, other_patterns, adt.shape.field_names()).map(
            |(pat1, pat2, names)| {
                let fat_arrow = fat_arrow();
                let body = match &*names {
                    [] => {
                        quote!(true)
                    }
                    [first, rest @ ..] => {
                        let rest = rest.iter().map(|it| {
                            let t1 = Ident::new(format!("{}_self", it.text), it.span);
                            let t2 = Ident::new(format!("{}_other", it.text), it.span);
                            let and_and = and_and();
                            quote!(#and_and #t1 .eq( #t2 ))
                        });
                        let first = {
                            let t1 = Ident::new(format!("{}_self", first.text), first.span);
                            let t2 = Ident::new(format!("{}_other", first.text), first.span);
                            quote!(#t1 .eq( #t2 ))
                        };
                        quote!(#first ##rest)
                    }
                };
                quote! { ( #pat1 , #pat2 ) #fat_arrow #body , }
            },
        );

        let fat_arrow = fat_arrow();
        quote! {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    ##arms
                    _unused #fat_arrow false
                }
            }
        }
    })
}

fn self_and_other_patterns(
    adt: &BasicAdtInfo,
    name: &tt::Ident,
) -> (Vec<tt::Subtree>, Vec<tt::Subtree>) {
    let self_patterns = adt.shape.as_pattern_map(name, |it| {
        let t = Ident::new(format!("{}_self", it.text), it.span);
        quote!(#t)
    });
    let other_patterns = adt.shape.as_pattern_map(name, |it| {
        let t = Ident::new(format!("{}_other", it.text), it.span);
        quote!(#t)
    });
    (self_patterns, other_patterns)
}

fn ord_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = &find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::cmp::Ord }, |adt| {
        fn compare(
            krate: &tt::TokenTree,
            left: tt::Subtree,
            right: tt::Subtree,
            rest: tt::Subtree,
        ) -> tt::Subtree {
            let fat_arrow1 = fat_arrow();
            let fat_arrow2 = fat_arrow();
            quote! {
                match #left.cmp(&#right) {
                    #krate::cmp::Ordering::Equal #fat_arrow1 {
                        #rest
                    }
                    c #fat_arrow2 return c,
                }
            }
        }
        if matches!(adt.shape, AdtShape::Union) {
            // FIXME: Return expand error here
            return quote!();
        }
        let (self_patterns, other_patterns) = self_and_other_patterns(adt, &adt.name);
        let arms = izip!(self_patterns, other_patterns, adt.shape.field_names()).map(
            |(pat1, pat2, fields)| {
                let mut body = quote!(#krate::cmp::Ordering::Equal);
                for f in fields.into_iter().rev() {
                    let t1 = Ident::new(format!("{}_self", f.text), f.span);
                    let t2 = Ident::new(format!("{}_other", f.text), f.span);
                    body = compare(krate, quote!(#t1), quote!(#t2), body);
                }
                let fat_arrow = fat_arrow();
                quote! { ( #pat1 , #pat2 ) #fat_arrow #body , }
            },
        );
        let fat_arrow = fat_arrow();
        let mut body = quote! {
            match (self, other) {
                ##arms
                _unused #fat_arrow #krate::cmp::Ordering::Equal
            }
        };
        if matches!(&adt.shape, AdtShape::Enum { .. }) {
            let left = quote!(#krate::intrinsics::discriminant_value(self));
            let right = quote!(#krate::intrinsics::discriminant_value(other));
            body = compare(krate, left, right, body);
        }
        quote! {
            fn cmp(&self, other: &Self) -> #krate::cmp::Ordering {
                #body
            }
        }
    })
}

fn partial_ord_expand(
    db: &dyn ExpandDatabase,
    id: MacroCallId,
    tt: &ast::Adt,
    tm: &TokenMap,
) -> ExpandResult<tt::Subtree> {
    let krate = &find_builtin_crate(db, id);
    expand_simple_derive(tt, tm, quote! { #krate::cmp::PartialOrd }, |adt| {
        fn compare(
            krate: &tt::TokenTree,
            left: tt::Subtree,
            right: tt::Subtree,
            rest: tt::Subtree,
        ) -> tt::Subtree {
            let fat_arrow1 = fat_arrow();
            let fat_arrow2 = fat_arrow();
            quote! {
                match #left.partial_cmp(&#right) {
                    #krate::option::Option::Some(#krate::cmp::Ordering::Equal) #fat_arrow1 {
                        #rest
                    }
                    c #fat_arrow2 return c,
                }
            }
        }
        if matches!(adt.shape, AdtShape::Union) {
            // FIXME: Return expand error here
            return quote!();
        }
        let left = quote!(#krate::intrinsics::discriminant_value(self));
        let right = quote!(#krate::intrinsics::discriminant_value(other));

        let (self_patterns, other_patterns) = self_and_other_patterns(adt, &adt.name);
        let arms = izip!(self_patterns, other_patterns, adt.shape.field_names()).map(
            |(pat1, pat2, fields)| {
                let mut body = quote!(#krate::option::Option::Some(#krate::cmp::Ordering::Equal));
                for f in fields.into_iter().rev() {
                    let t1 = Ident::new(format!("{}_self", f.text), f.span);
                    let t2 = Ident::new(format!("{}_other", f.text), f.span);
                    body = compare(krate, quote!(#t1), quote!(#t2), body);
                }
                let fat_arrow = fat_arrow();
                quote! { ( #pat1 , #pat2 ) #fat_arrow #body , }
            },
        );
        let fat_arrow = fat_arrow();
        let body = compare(
            krate,
            left,
            right,
            quote! {
                match (self, other) {
                    ##arms
                    _unused #fat_arrow #krate::option::Option::Some(#krate::cmp::Ordering::Equal)
                }
            },
        );
        quote! {
            fn partial_cmp(&self, other: &Self) -> #krate::option::Option::Option<#krate::cmp::Ordering> {
                #body
            }
        }
    })
}
