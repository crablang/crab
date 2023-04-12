//! This module has the functionality to search the project and its dependencies for a certain item,
//! by its name and a few criteria.
//! The main reason for this module to exist is the fact that project's items and dependencies' items
//! are located in different caches, with different APIs.
use either::Either;
use hir::{
    import_map::{self, ImportKind},
    symbols::FileSymbol,
    AsAssocItem, Crate, ItemInNs, Semantics,
};
use limit::Limit;
use syntax::{ast, AstNode, SyntaxKind::NAME};

use crate::{
    defs::{Definition, NameClass},
    imports::import_assets::NameToImport,
    symbol_index, RootDatabase,
};

/// A value to use, when uncertain which limit to pick.
pub static DEFAULT_QUERY_SEARCH_LIMIT: Limit = Limit::new(40);

/// Three possible ways to search for the name in associated and/or other items.
#[derive(Debug, Clone, Copy)]
pub enum AssocItemSearch {
    /// Search for the name in both associated and other items.
    Include,
    /// Search for the name in other items only.
    Exclude,
    /// Search for the name in the associated items only.
    AssocItemsOnly,
}

/// Searches for importable items with the given name in the crate and its dependencies.
pub fn items_with_name<'a>(
    sema: &'a Semantics<'_, RootDatabase>,
    krate: Crate,
    name: NameToImport,
    assoc_item_search: AssocItemSearch,
    limit: Option<usize>,
) -> impl Iterator<Item = ItemInNs> + 'a {
    let _p = profile::span("items_with_name").detail(|| {
        format!(
            "Name: {}, crate: {:?}, assoc items: {:?}, limit: {:?}",
            name.text(),
            assoc_item_search,
            krate.display_name(sema.db).map(|name| name.to_string()),
            limit,
        )
    });

    let (mut local_query, mut external_query) = match name {
        NameToImport::Exact(exact_name, case_sensitive) => {
            let mut local_query = symbol_index::Query::new(exact_name.clone());
            local_query.exact();

            let external_query = import_map::Query::new(exact_name)
                .name_only()
                .search_mode(import_map::SearchMode::Equals);

            (
                local_query,
                if case_sensitive { external_query.case_sensitive() } else { external_query },
            )
        }
        NameToImport::Fuzzy(fuzzy_search_string) => {
            let mut local_query = symbol_index::Query::new(fuzzy_search_string.clone());

            let mut external_query = import_map::Query::new(fuzzy_search_string.clone())
                .search_mode(import_map::SearchMode::Fuzzy)
                .name_only();
            match assoc_item_search {
                AssocItemSearch::Include => {}
                AssocItemSearch::Exclude => {
                    external_query = external_query.exclude_import_kind(ImportKind::AssociatedItem);
                }
                AssocItemSearch::AssocItemsOnly => {
                    external_query = external_query.assoc_items_only();
                }
            }

            if fuzzy_search_string.to_lowercase() != fuzzy_search_string {
                local_query.case_sensitive();
                external_query = external_query.case_sensitive();
            }

            (local_query, external_query)
        }
    };

    if let Some(limit) = limit {
        external_query = external_query.limit(limit);
        local_query.limit(limit);
    }

    find_items(sema, krate, assoc_item_search, local_query, external_query)
}

fn find_items<'a>(
    sema: &'a Semantics<'_, RootDatabase>,
    krate: Crate,
    assoc_item_search: AssocItemSearch,
    local_query: symbol_index::Query,
    external_query: import_map::Query,
) -> impl Iterator<Item = ItemInNs> + 'a {
    let _p = profile::span("find_items");
    let db = sema.db;

    let external_importables =
        krate.query_external_importables(db, external_query).map(|external_importable| {
            match external_importable {
                Either::Left(module_def) => ItemInNs::from(module_def),
                Either::Right(macro_def) => ItemInNs::from(macro_def),
            }
        });

    // Query the local crate using the symbol index.
    let local_results = symbol_index::crate_symbols(db, krate, local_query)
        .into_iter()
        .filter_map(move |local_candidate| get_name_definition(sema, &local_candidate))
        .filter_map(|name_definition_to_import| match name_definition_to_import {
            Definition::Macro(macro_def) => Some(ItemInNs::from(macro_def)),
            def => <Option<_>>::from(def),
        });

    external_importables.chain(local_results).filter(move |&item| match assoc_item_search {
        AssocItemSearch::Include => true,
        AssocItemSearch::Exclude => !is_assoc_item(item, sema.db),
        AssocItemSearch::AssocItemsOnly => is_assoc_item(item, sema.db),
    })
}

fn get_name_definition(
    sema: &Semantics<'_, RootDatabase>,
    import_candidate: &FileSymbol,
) -> Option<Definition> {
    let _p = profile::span("get_name_definition");

    let candidate_node = import_candidate.loc.syntax(sema)?;
    let candidate_name_node = if candidate_node.kind() != NAME {
        candidate_node.children().find(|it| it.kind() == NAME)?
    } else {
        candidate_node
    };
    let name = ast::Name::cast(candidate_name_node)?;
    NameClass::classify(sema, &name)?.defined()
}

fn is_assoc_item(item: ItemInNs, db: &RootDatabase) -> bool {
    item.as_module_def().and_then(|module_def| module_def.as_assoc_item(db)).is_some()
}
