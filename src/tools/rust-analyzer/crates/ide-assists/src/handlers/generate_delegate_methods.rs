use std::collections::HashSet;

use hir::{self, HasCrate, HasSource, HasVisibility};
use syntax::{
    ast::{
        self, edit_in_place::Indent, make, AstNode, HasGenericParams, HasName, HasVisibility as _,
    },
    ted,
};

use crate::{
    utils::{convert_param_list_to_arg_list, find_struct_impl},
    AssistContext, AssistId, AssistKind, Assists, GroupLabel,
};

// Assist: generate_delegate_methods
//
// Generate delegate methods.
//
// ```
// struct Age(u8);
// impl Age {
//     fn age(&self) -> u8 {
//         self.0
//     }
// }
//
// struct Person {
//     ag$0e: Age,
// }
// ```
// ->
// ```
// struct Age(u8);
// impl Age {
//     fn age(&self) -> u8 {
//         self.0
//     }
// }
//
// struct Person {
//     age: Age,
// }
//
// impl Person {
//     $0fn age(&self) -> u8 {
//         self.age.age()
//     }
// }
// ```
pub(crate) fn generate_delegate_methods(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let strukt = ctx.find_node_at_offset::<ast::Struct>()?;
    let strukt_name = strukt.name()?;
    let current_module = ctx.sema.scope(strukt.syntax())?.module();

    let (field_name, field_ty, target) = match ctx.find_node_at_offset::<ast::RecordField>() {
        Some(field) => {
            let field_name = field.name()?;
            let field_ty = field.ty()?;
            (field_name.to_string(), field_ty, field.syntax().text_range())
        }
        None => {
            let field = ctx.find_node_at_offset::<ast::TupleField>()?;
            let field_list = ctx.find_node_at_offset::<ast::TupleFieldList>()?;
            let field_list_index = field_list.fields().position(|it| it == field)?;
            let field_ty = field.ty()?;
            (field_list_index.to_string(), field_ty, field.syntax().text_range())
        }
    };

    let sema_field_ty = ctx.sema.resolve_type(&field_ty)?;
    let mut methods = vec![];
    let mut seen_names = HashSet::new();

    for ty in sema_field_ty.autoderef(ctx.db()) {
        let krate = ty.krate(ctx.db());
        ty.iterate_assoc_items(ctx.db(), krate, |item| {
            if let hir::AssocItem::Function(f) = item {
                let name = f.name(ctx.db());
                if f.self_param(ctx.db()).is_some()
                    && f.is_visible_from(ctx.db(), current_module)
                    && seen_names.insert(name.clone())
                {
                    methods.push((name, f))
                }
            }
            Option::<()>::None
        });
    }
    methods.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (name, method) in methods {
        let adt = ast::Adt::Struct(strukt.clone());
        let name = name.display(ctx.db()).to_string();
        // if `find_struct_impl` returns None, that means that a function named `name` already exists.
        let Some(impl_def) = find_struct_impl(ctx, &adt, std::slice::from_ref(&name)) else {
            continue;
        };
        acc.add_group(
            &GroupLabel("Generate delegate methods…".to_owned()),
            AssistId("generate_delegate_methods", AssistKind::Generate),
            format!("Generate delegate for `{field_name}.{name}()`",),
            target,
            |edit| {
                // Create the function
                let method_source = match method.source(ctx.db()) {
                    Some(source) => source.value,
                    None => return,
                };
                let vis = method_source.visibility();
                let fn_name = make::name(&name);
                let params =
                    method_source.param_list().unwrap_or_else(|| make::param_list(None, []));
                let type_params = method_source.generic_param_list();
                let arg_list = match method_source.param_list() {
                    Some(list) => convert_param_list_to_arg_list(list),
                    None => make::arg_list([]),
                };
                let tail_expr = make::expr_method_call(
                    make::ext::field_from_idents(["self", &field_name]).unwrap(), // This unwrap is ok because we have at least 1 arg in the list
                    make::name_ref(&name),
                    arg_list,
                );
                let ret_type = method_source.ret_type();
                let is_async = method_source.async_token().is_some();
                let is_const = method_source.const_token().is_some();
                let is_unsafe = method_source.unsafe_token().is_some();
                let tail_expr_finished =
                    if is_async { make::expr_await(tail_expr) } else { tail_expr };
                let body = make::block_expr([], Some(tail_expr_finished));
                let f = make::fn_(
                    vis,
                    fn_name,
                    type_params,
                    None,
                    params,
                    body,
                    ret_type,
                    is_async,
                    is_const,
                    is_unsafe,
                )
                .clone_for_update();

                // Get the impl to update, or create one if we need to.
                let impl_def = match impl_def {
                    Some(impl_def) => edit.make_mut(impl_def),
                    None => {
                        let name = &strukt_name.to_string();
                        let params = strukt.generic_param_list();
                        let ty_params = params.clone();
                        let where_clause = strukt.where_clause();

                        let impl_def = make::impl_(
                            ty_params,
                            None,
                            make::ty_path(make::ext::ident_path(name)),
                            where_clause,
                            None,
                        )
                        .clone_for_update();

                        // Fixup impl_def indentation
                        let indent = strukt.indent_level();
                        impl_def.reindent_to(indent);

                        // Insert the impl block.
                        let strukt = edit.make_mut(strukt.clone());
                        ted::insert_all(
                            ted::Position::after(strukt.syntax()),
                            vec![
                                make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                                impl_def.syntax().clone().into(),
                            ],
                        );

                        impl_def
                    }
                };

                // Fixup function indentation.
                // FIXME: Should really be handled by `AssocItemList::add_item`
                f.reindent_to(impl_def.indent_level() + 1);

                let assoc_items = impl_def.get_or_create_assoc_item_list();
                assoc_items.add_item(f.clone().into());

                if let Some(cap) = ctx.config.snippet_cap {
                    edit.add_tabstop_before(cap, f)
                }
            },
        )?;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_generate_delegate_create_impl_block() {
        check_assist(
            generate_delegate_methods,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person {
    ag$0e: Age,
}"#,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person {
    age: Age,
}

impl Person {
    $0fn age(&self) -> u8 {
        self.age.age()
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_create_impl_block_match_indent() {
        check_assist(
            generate_delegate_methods,
            r#"
mod indent {
    struct Age(u8);
    impl Age {
        fn age(&self) -> u8 {
            self.0
        }
    }

    struct Person {
        ag$0e: Age,
    }
}"#,
            r#"
mod indent {
    struct Age(u8);
    impl Age {
        fn age(&self) -> u8 {
            self.0
        }
    }

    struct Person {
        age: Age,
    }

    impl Person {
        $0fn age(&self) -> u8 {
            self.age.age()
        }
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_update_impl_block() {
        check_assist(
            generate_delegate_methods,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person {
    ag$0e: Age,
}

impl Person {}"#,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person {
    age: Age,
}

impl Person {
    $0fn age(&self) -> u8 {
        self.age.age()
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_update_impl_block_match_indent() {
        check_assist(
            generate_delegate_methods,
            r#"
mod indent {
    struct Age(u8);
    impl Age {
        fn age(&self) -> u8 {
            self.0
        }
    }

    struct Person {
        ag$0e: Age,
    }

    impl Person {}
}"#,
            r#"
mod indent {
    struct Age(u8);
    impl Age {
        fn age(&self) -> u8 {
            self.0
        }
    }

    struct Person {
        age: Age,
    }

    impl Person {
        $0fn age(&self) -> u8 {
            self.age.age()
        }
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_tuple_struct() {
        check_assist(
            generate_delegate_methods,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person(A$0ge);"#,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person(Age);

impl Person {
    $0fn age(&self) -> u8 {
        self.0.age()
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_enable_all_attributes() {
        check_assist(
            generate_delegate_methods,
            r#"
struct Age<T>(T);
impl<T> Age<T> {
    pub(crate) async fn age<J, 'a>(&'a mut self, ty: T, arg: J) -> T {
        self.0
    }
}

struct Person<T> {
    ag$0e: Age<T>,
}"#,
            r#"
struct Age<T>(T);
impl<T> Age<T> {
    pub(crate) async fn age<J, 'a>(&'a mut self, ty: T, arg: J) -> T {
        self.0
    }
}

struct Person<T> {
    age: Age<T>,
}

impl<T> Person<T> {
    $0pub(crate) async fn age<J, 'a>(&'a mut self, ty: T, arg: J) -> T {
        self.age.age(ty, arg).await
    }
}"#,
        );
    }

    #[test]
    fn test_generates_delegate_autoderef() {
        check_assist(
            generate_delegate_methods,
            r#"
//- minicore: deref
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}
struct AgeDeref(Age);
impl core::ops::Deref for AgeDeref { type Target = Age; }
struct Person {
    ag$0e: AgeDeref,
}
impl Person {}"#,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}
struct AgeDeref(Age);
impl core::ops::Deref for AgeDeref { type Target = Age; }
struct Person {
    age: AgeDeref,
}
impl Person {
    $0fn age(&self) -> u8 {
        self.age.age()
    }
}"#,
        );
    }

    #[test]
    fn test_generate_delegate_visibility() {
        check_assist_not_applicable(
            generate_delegate_methods,
            r#"
mod m {
    pub struct Age(u8);
    impl Age {
        fn age(&self) -> u8 {
            self.0
        }
    }
}

struct Person {
    ag$0e: m::Age,
}"#,
        )
    }

    #[test]
    fn test_generate_not_eligible_if_fn_exists() {
        check_assist_not_applicable(
            generate_delegate_methods,
            r#"
struct Age(u8);
impl Age {
    fn age(&self) -> u8 {
        self.0
    }
}

struct Person {
    ag$0e: Age,
}
impl Person {
    fn age(&self) -> u8 { 0 }
}
"#,
        );
    }
}
