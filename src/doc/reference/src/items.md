# Items

> **<sup>Syntax:<sup>**\
> _Item_:\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; _VisItem_\
> &nbsp;&nbsp; | _MacroItem_
>
> _VisItem_:\
> &nbsp;&nbsp; [_Visibility_]<sup>?</sup>\
> &nbsp;&nbsp; (\
> &nbsp;&nbsp; &nbsp;&nbsp; &nbsp;&nbsp;  [_Module_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_ExternCrate_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_UseDeclaration_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Function_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_TypeAlias_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Struct_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Enumeration_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Union_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_ConstantItem_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_StaticItem_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Trait_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_Implementation_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_ExternBlock_]\
> &nbsp;&nbsp; )
>
> _MacroItem_:\
> &nbsp;&nbsp; &nbsp;&nbsp; [_MacroInvocationSemi_]\
> &nbsp;&nbsp; | [_MacroRulesDefinition_]


An _item_ is a component of a crate. Items are organized within a crate by a
nested set of [modules]. Every crate has a single "outermost" anonymous module;
all further items within the crate have [paths] within the module tree of the
crate.

Items are entirely determined at compile-time, generally remain fixed during
execution, and may reside in read-only memory.

There are several kinds of items:

* [modules]
* [`extern crate` declarations]
* [`use` declarations]
* [function definitions]
* [type definitions]
* [struct definitions]
* [enumeration definitions]
* [union definitions]
* [constant items]
* [static items]
* [trait definitions]
* [implementations]
* [`extern` blocks]

Items may be declared in the [root of the crate], a [module][modules], or a [block expression].
A subset of items, called [associated items], may be declared in [traits] and [implementations].
A subset of items, called external items, may be declared in [`extern` blocks].

Items may be defined in any order, with the exception of [`macro_rules`] which has its own scoping behavior.
[Name resolution] of item names allows items to be defined before or after where the item is referred to in the module or block.

See [item scopes] for information on the scoping rules of items.

[_ConstantItem_]: items/constant-items.md
[_Enumeration_]: items/enumerations.md
[_ExternBlock_]: items/external-blocks.md
[_ExternCrate_]: items/extern-crates.md
[_Function_]: items/functions.md
[_Implementation_]: items/implementations.md
[_MacroInvocationSemi_]: macros.md#macro-invocation
[_MacroRulesDefinition_]: macros-by-example.md
[_Module_]: items/modules.md
[_OuterAttribute_]: attributes.md
[_StaticItem_]: items/static-items.md
[_Struct_]: items/structs.md
[_Trait_]: items/traits.md
[_TypeAlias_]: items/type-aliases.md
[_Union_]: items/unions.md
[_UseDeclaration_]: items/use-declarations.md
[_Visibility_]: visibility-and-privacy.md
[`extern crate` declarations]: items/extern-crates.md
[`extern` blocks]: items/external-blocks.md
[`macro_rules`]: macros-by-example.md
[`use` declarations]: items/use-declarations.md
[associated items]: items/associated-items.md
[block expression]: expressions/block-expr.md
[constant items]: items/constant-items.md
[enumeration definitions]: items/enumerations.md
[function definitions]: items/functions.md
[implementations]: items/implementations.md
[item scopes]: names/scopes.md#item-scopes
[modules]: items/modules.md
[name resolution]: names/name-resolution.md
[paths]: paths.md
[root of the crate]: crates-and-source-files.md
[statement]: statements.md
[static items]: items/static-items.md
[struct definitions]: items/structs.md
[trait definitions]: items/traits.md
[traits]: items/traits.md
[type definitions]: items/type-aliases.md
[union definitions]: items/unions.md
