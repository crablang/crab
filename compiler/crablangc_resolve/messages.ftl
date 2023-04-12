resolve_parent_module_reset_for_binding =
    parent module is reset for binding

resolve_ampersand_used_without_explicit_lifetime_name =
    `&` without an explicit lifetime name cannot be used here
    .note = explicit lifetime name needed here

resolve_underscore_lifetime_name_cannot_be_used_here =
    `'_` cannot be used here
    .note = `'_` is a reserved lifetime name

resolve_crate_may_not_be_imported =
    `$crate` may not be imported

resolve_crate_root_imports_must_be_named_explicitly =
    crate root imports need to be explicitly named: `use crate as name;`

resolve_generic_params_from_outer_function =
    can't use generic parameters from outer function
    .label = use of generic parameter from outer function
    .suggestion = try using a local generic parameter instead

resolve_self_type_implicitly_declared_by_impl =
    `Self` type implicitly declared here, by this `impl`

resolve_cannot_use_self_type_here =
    can't use `Self` here

resolve_use_a_type_here_instead =
    use a type here instead

resolve_type_param_from_outer_fn =
    type parameter from outer function

resolve_const_param_from_outer_fn =
    const parameter from outer function

resolve_try_using_local_generic_parameter =
    try using a local generic parameter instead

resolve_try_adding_local_generic_param_on_method =
    try adding a local generic parameter in this method instead

resolve_help_try_using_local_generic_param =
    try using a local generic paramter instead

resolve_name_is_already_used_as_generic_parameter =
    the name `{$name}` is already used for a generic parameter in this item's generic parameters
    .label = already used
    .first_use_of_name = first use of `{$name}`

resolve_method_not_member_of_trait =
    method `{$method}` is not a member of trait `{$trait_}`
    .label = not a member of trait `{$trait_}`

resolve_associated_fn_with_similar_name_exists =
    there is an associated function with a similar name

resolve_type_not_member_of_trait =
    type `{$type_}` is not a member of trait `{$trait_}`
    .label = not a member of trait `{$trait_}`

resolve_associated_type_with_similar_name_exists =
    there is an associated type with a similar name

resolve_const_not_member_of_trait =
    const `{$const_}` is not a member of trait `{$trait_}`
    .label = not a member of trait `{$trait_}`

resolve_associated_const_with_similar_name_exists =
    there is an associated constant with a similar name

resolve_variable_bound_with_different_mode =
    variable `{$variable_name}` is bound inconsistently across alternatives separated by `|`
    .label = bound in different ways
    .first_binding_span = first binding

resolve_ident_bound_more_than_once_in_parameter_list =
    identifier `{$identifier}` is bound more than once in this parameter list
    .label = used as parameter more than once

resolve_ident_bound_more_than_once_in_same_pattern =
    identifier `{$identifier}` is bound more than once in the same pattern
    .label = used in a pattern more than once

resolve_undeclared_label =
    use of undeclared label `{$name}`
    .label = undeclared label `{$name}`

resolve_label_with_similar_name_reachable =
    a label with a similar name is reachable

resolve_try_using_similarly_named_label =
    try using similarly named label

resolve_unreachable_label_with_similar_name_exists =
    a label with a similar name exists but is unreachable

resolve_self_import_can_only_appear_once_in_the_list =
    `self` import can only appear once in an import list
    .label = can only appear once in an import list

resolve_self_import_only_in_import_list_with_non_empty_prefix =
    `self` import can only appear in an import list with a non-empty prefix
    .label = can only appear in an import list with a non-empty prefix

resolve_cannot_capture_dynamic_environment_in_fn_item =
    can't capture dynamic environment in a fn item
    .help = use the `|| {"{"} ... {"}"}` closure form instead

resolve_attempt_to_use_non_constant_value_in_constant =
    attempt to use a non-constant value in a constant

resolve_attempt_to_use_non_constant_value_in_constant_with_suggestion =
    consider using `{$suggestion}` instead of `{$current}`

resolve_attempt_to_use_non_constant_value_in_constant_label_with_suggestion =
    non-constant value

resolve_attempt_to_use_non_constant_value_in_constant_without_suggestion =
    this would need to be a `{$suggestion}`

resolve_self_imports_only_allowed_within =
    `self` imports are only allowed within a {"{"} {"}"} list

resolve_self_imports_only_allowed_within_suggestion =
    consider importing the module directly

resolve_self_imports_only_allowed_within_multipart_suggestion =
    alternatively, use the multi-path `use` syntax to import `self`

resolve_binding_shadows_something_unacceptable =
    {$shadowing_binding}s cannot shadow {$shadowed_binding}s
    .label = cannot be named the same as {$article} {$shadowed_binding}
    .label_shadowed_binding = the {$shadowed_binding} `{$name}` is {$participle} here

resolve_binding_shadows_something_unacceptable_suggestion =
    try specify the pattern arguments

resolve_forward_declared_generic_param =
    generic parameters with a default cannot use forward declared identifiers
    .label = defaulted generic parameters cannot be forward declared

resolve_param_in_ty_of_const_param =
    the type of const parameters must not depend on other generic parameters
    .label = the type must not depend on the parameter `{$name}`

resolve_self_in_generic_param_default =
    generic parameters cannot use `Self` in their defaults
    .label = `Self` in generic parameter default

resolve_param_in_non_trivial_anon_const =
    generic parameters may not be used in const operations
    .label = cannot perform const operation using `{$name}`

resolve_param_in_non_trivial_anon_const_help =
    use `#![feature(generic_const_exprs)]` to allow generic const expressions

resolve_param_in_non_trivial_anon_const_sub_type =
    type parameters may not be used in const expressions

resolve_param_in_non_trivial_anon_const_sub_non_type =
    const parameters may only be used as standalone arguments, i.e. `{$name}`

resolve_unreachable_label =
    use of unreachable label `{$name}`
    .label = unreachable label `{$name}`
    .label_definition_span = unreachable label defined here
    .note = labels are unreachable through functions, closures, async blocks and modules

resolve_unreachable_label_suggestion_use_similarly_named =
    try using similarly named label

resolve_unreachable_label_similar_name_reachable =
    a label with a similar name is reachable

resolve_unreachable_label_similar_name_unreachable =
    a label with a similar name exists but is also unreachable

resolve_trait_impl_mismatch =
    item `{$name}` is an associated {$kind}, which doesn't match its trait `{$trait_path}`
    .label = does not match trait
    .label_trait_item = item in trait

resolve_invalid_asm_sym =
    invalid `sym` operand
    .label = is a local variable
    .help = `sym` operands must refer to either a function or a static

resolve_trait_impl_duplicate =
    duplicate definitions with name `{$name}`:
    .label = duplicate definition
    .old_span_label = previous definition here
    .trait_item_span = item in trait

resolve_relative_2018 =
    relative paths are not supported in visibilities in 2018 edition or later
    .suggestion = try

resolve_ancestor_only =
    visibilities can only be restricted to ancestor modules

resolve_expected_found =
    expected module, found {$res} `{$path_str}`
    .label = not a module

resolve_indeterminate =
    cannot determine resolution for the visibility

resolve_tool_module_imported =
    cannot use a tool module through an import
    .note = the tool module imported here

resolve_module_only =
    visibility must resolve to a module

resolve_macro_expected_found =
    expected {$expected}, found {$found} `{$macro_path}`

resolve_remove_surrounding_derive =
    remove from the surrounding `derive()`

resolve_add_as_non_derive =
    add as non-Derive macro
    `#[{$macro_path}]`
