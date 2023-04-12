//! This file provides snippet completions, like `pd` => `eprintln!(...)`.

use hir::Documentation;
use ide_db::{imports::insert_use::ImportScope, SnippetCap};

use crate::{
    context::{ExprCtx, ItemListKind, PathCompletionCtx, Qualified},
    item::Builder,
    CompletionContext, CompletionItem, CompletionItemKind, Completions, SnippetScope,
};

pub(crate) fn complete_expr_snippet(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    path_ctx: &PathCompletionCtx,
    &ExprCtx { in_block_expr, .. }: &ExprCtx,
) {
    if !matches!(path_ctx.qualified, Qualified::No) {
        return;
    }
    if !ctx.qualifier_ctx.none() {
        return;
    }

    let cap = match ctx.config.snippet_cap {
        Some(it) => it,
        None => return,
    };

    if !ctx.config.snippets.is_empty() {
        add_custom_completions(acc, ctx, cap, SnippetScope::Expr);
    }

    if in_block_expr {
        snippet(ctx, cap, "pd", "eprintln!(\"$0 = {:?}\", $0);").add_to(acc);
        snippet(ctx, cap, "ppd", "eprintln!(\"$0 = {:#?}\", $0);").add_to(acc);
        let item = snippet(
            ctx,
            cap,
            "macro_rules",
            "\
macro_rules! $1 {
    ($2) => {
        $0
    };
}",
        );
        item.add_to(acc);
    }
}

pub(crate) fn complete_item_snippet(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    path_ctx: &PathCompletionCtx,
    kind: &ItemListKind,
) {
    if !matches!(path_ctx.qualified, Qualified::No) {
        return;
    }
    if !ctx.qualifier_ctx.none() {
        return;
    }
    let cap = match ctx.config.snippet_cap {
        Some(it) => it,
        None => return,
    };

    if !ctx.config.snippets.is_empty() {
        add_custom_completions(acc, ctx, cap, SnippetScope::Item);
    }

    // Test-related snippets shouldn't be shown in blocks.
    if let ItemListKind::SourceFile | ItemListKind::Module = kind {
        let mut item = snippet(
            ctx,
            cap,
            "tmod (Test module)",
            "\
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ${1:test_name}() {
        $0
    }
}",
        );
        item.lookup_by("tmod");
        item.add_to(acc);

        let mut item = snippet(
            ctx,
            cap,
            "tfn (Test function)",
            "\
#[test]
fn ${1:feature}() {
    $0
}",
        );
        item.lookup_by("tfn");
        item.add_to(acc);

        let item = snippet(
            ctx,
            cap,
            "macro_rules",
            "\
macro_rules! $1 {
    ($2) => {
        $0
    };
}",
        );
        item.add_to(acc);
    }
}

fn snippet(ctx: &CompletionContext<'_>, cap: SnippetCap, label: &str, snippet: &str) -> Builder {
    let mut item = CompletionItem::new(CompletionItemKind::Snippet, ctx.source_range(), label);
    item.insert_snippet(cap, snippet);
    item
}

fn add_custom_completions(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    cap: SnippetCap,
    scope: SnippetScope,
) -> Option<()> {
    if ImportScope::find_insert_use_container(&ctx.token.parent()?, &ctx.sema).is_none() {
        return None;
    }
    ctx.config.prefix_snippets().filter(|(_, snip)| snip.scope == scope).for_each(
        |(trigger, snip)| {
            let imports = match snip.imports(ctx) {
                Some(imports) => imports,
                None => return,
            };
            let body = snip.snippet();
            let mut builder = snippet(ctx, cap, trigger, &body);
            builder.documentation(Documentation::new(format!("```crablang\n{body}\n```")));
            for import in imports.into_iter() {
                builder.add_import(import);
            }
            builder.set_detail(snip.description.clone());
            builder.add_to(acc);
        },
    );
    None
}

#[cfg(test)]
mod tests {
    use crate::{
        tests::{check_edit_with_config, TEST_CONFIG},
        CompletionConfig, Snippet,
    };

    #[test]
    fn custom_snippet_completion() {
        check_edit_with_config(
            CompletionConfig {
                snippets: vec![Snippet::new(
                    &["break".into()],
                    &[],
                    &["ControlFlow::Break(())".into()],
                    "",
                    &["core::ops::ControlFlow".into()],
                    crate::SnippetScope::Expr,
                )
                .unwrap()],
                ..TEST_CONFIG
            },
            "break",
            r#"
//- minicore: try
fn main() { $0 }
"#,
            r#"
use core::ops::ControlFlow;

fn main() { ControlFlow::Break(()) }
"#,
        );
    }
}
