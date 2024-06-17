# Scopes

A *scope* is the region of source text where a named [entity] may be referenced with that name.
The following sections provide details on the scoping rules and behavior, which depend on the kind of entity and where it is declared.
The process of how names are resolved to entities is described in the [name resolution] chapter.
More information on "drop scopes" used for the purpose of running destructors may be found in the [destructors] chapter.

## Item scopes

The name of an [item][items] declared directly in a [module] has a scope that extends from the start of the module to the end of the module. These items are also members of the module and can be referred to with a [path] leading from their module.

The name of an item declared as a [statement] has a scope that extends from the start of the block the item statement is in until the end of the block.

It is an error to introduce an item with a duplicate name of another item in the same [namespace] within the same module or block.
[Asterisk glob imports] have special behavior for dealing with duplicate names and shadowing, see the linked chapter for more details.
Items in a module may shadow items in a [prelude](#prelude-scopes).

Item names from outer modules are not in scope within a nested module.
A [path] may be used to refer to an item in another module.

### Associated item scopes

[Associated items] are not scoped and can only be referred to by using a [path] leading from the type or trait they are associated with.
[Methods] can also be referred to via [call expressions].

Similar to items within a module or block,  it is an error to introduce an item within a trait or implementation that is a duplicate of another item in the trait or impl in the same namespace.

## Pattern binding scopes

The scope of a local variable [pattern] binding depends on where it is used:

* [`let` statement] bindings range from just after the `let` statement until the end of the block where it is declared.
* [Function parameter] bindings are within the body of the function.
* [Closure parameter] bindings are within the closure body.
* [`for`] and [`while let`] bindings are within the loop body.
* [`if let`] bindings are within the consequent block.
* [`match` arms] bindings are within the [match guard] and the match arm expression.

Local variable scopes do not extend into item declarations.
<!-- Not entirely, see https://github.com/rust-lang/rust/issues/33118 -->

### Pattern binding shadowing

Pattern bindings are allowed to shadow any name in scope with the following exceptions which are an error:

* [Const generic parameters]
* [Static items]
* [Const items]
* Constructors for [structs] and [enums]

The following example illustrates how local bindings can shadow item declarations:

```rust
fn shadow_example() {
    // Since there are no local variables in scope yet, this resolves to the function.
    foo(); // prints `function`
    let foo = || println!("closure");
    fn foo() { println!("function"); }
    // This resolves to the local closure since it shadows the item.
    foo(); // prints `closure`
}
```

## Generic parameter scopes

Generic parameters are declared in a [_GenericParams_] list.
The scope of a generic parameter is within the item it is declared on.

All parameters are in scope within the generic parameter list regardless of the order they are declared.
The following shows some examples where a parameter may be referenced before it is declared:

```rust
// The 'b bound is referenced before it is declared.
fn params_scope<'a: 'b, 'b>() {}

# trait SomeTrait<const Z: usize> {}
// The const N is referenced in the trait bound before it is declared.
fn f<T: SomeTrait<N>, const N: usize>() {}
```

Generic parameters are also in scope for type bounds and where clauses, for example:

```rust
# trait SomeTrait<'a, T> {}
// The <'a, U> for `SomeTrait` refer to the 'a and U parameters of `bounds_scope`.
fn bounds_scope<'a, T: SomeTrait<'a, U>, U>() {}

fn where_scope<'a, T, U>()
    where T: SomeTrait<'a, U>
{}
```

It is an error for [items] declared inside a function to refer to a generic parameter from their outer scope.

```rust,compile_fail
fn example<T>() {
    fn inner(x: T) {} // ERROR: can't use generic parameters from outer function
}
```

### Generic parameter shadowing

It is an error to shadow a generic parameter with the exception that items declared within functions are allowed to shadow generic parameter names from the function.

```rust
fn example<'a, T, const N: usize>() {
    // Items within functions are allowed to shadow generic parameter in scope.
    fn inner_lifetime<'a>() {} // OK
    fn inner_type<T>() {} // OK
    fn inner_const<const N: usize>() {} // OK
}
```

```rust,compile_fail
trait SomeTrait<'a, T, const N: usize> {
    fn example_lifetime<'a>() {} // ERROR: 'a is already in use
    fn example_type<T>() {} // ERROR: T is already in use
    fn example_const<const N: usize>() {} // ERROR: N is already in use
    fn example_mixed<const T: usize>() {} // ERROR: T is already in use
}
```

### Lifetime scopes

Lifetime parameters are declared in a [_GenericParams_] list and [higher-ranked trait bounds][hrtb].

The `'static` lifetime and [placeholder lifetime] `'_` have a special meaning and cannot be declared as a parameter.

#### Lifetime generic parameter scopes

[Constant] and [static] items and [const contexts] only ever allow `'static` lifetime references, so no other lifetime may be in scope within them.
[Associated consts] do allow referring to lifetimes declared in their trait or implementation.

#### Higher-ranked trait bound scopes

The scope of a lifetime parameter declared as a [higher-ranked trait bound][hrtb] depends on the scenario where it is used.

* As a [_TypeBoundWhereClauseItem_] the declared lifetimes are in scope in the type and the type bounds.
* As a [_TraitBound_] the declared lifetimes are in scope within the bound type path.
* As a [_BareFunctionType_] the declared lifetimes are in scope within the function parameters and return type.

```rust
# trait Trait<'a>{}

fn where_clause<T>()
    // 'a is in scope in both the type and the type bounds.
    where for <'a> &'a T: Trait<'a>
{}

fn bound<T>()
    // 'a is in scope within the bound.
    where T: for <'a> Trait<'a>
{}

# struct Example<'a> {
#     field: &'a u32
# }

// 'a is in scope in both the parameters and return type.
type FnExample = for<'a> fn(x: Example<'a>) -> Example<'a>;
```

#### Impl trait restrictions

[Impl trait] types can only reference lifetimes declared on a function or implementation.

<!-- not able to demonstrate the scope error because the compiler panics
     https://github.com/rust-lang/rust/issues/67830
-->
```rust
# trait Trait1 {
#     type Item;
# }
# trait Trait2<'a> {}
#
# struct Example;
#
# impl Trait1 for Example {
#     type Item = Element;
# }
#
# struct Element;
# impl<'a> Trait2<'a> for Element {}
#
// The `impl Trait2` here is not allowed to refer to 'b but it is allowed to
// refer to 'a.
fn foo<'a>() -> impl for<'b> Trait1<Item = impl Trait2<'a>> {
    // ...
#    Example
}
```

## Loop label scopes

[Loop labels] may be declared by a [loop expression].
The scope of a loop label is from the point it is declared till the end of the loop expression.
The scope does not extend into [items], [closures], [async blocks], [const arguments], [const contexts], and the iterator expression of the defining [`for` loop].

```rust
'a: for n in 0..3 {
    if n % 2 == 0 {
        break 'a;
    }
    fn inner() {
        // Using 'a here would be an error.
        // break 'a;
    }
}

// The label is in scope for the expression of `while` loops.
'a: while break 'a {}         // Loop does not run.
'a: while let _ = break 'a {} // Loop does not run.

// The label is not in scope in the defining `for` loop:
'a: for outer in 0..5 {
    // This will break the outer loop, skipping the inner loop and stopping
    // the outer loop.
    'a: for inner in { break 'a; 0..1 } {
        println!("{}", inner); // This does not run.
    }
    println!("{}", outer); // This does not run, either.
}

```

Loop labels may shadow labels of the same name in outer scopes.
References to a label refer to the closest definition.

```rust
// Loop label shadowing example.
'a: for outer in 0..5 {
    'a: for inner in 0..5 {
        // This terminates the inner loop, but the outer loop continues to run.
        break 'a;
    }
}
```

## Prelude scopes

[Preludes] bring entities into scope of every module.
The entities are not members of the module, but are implicitly queried during [name resolution].
The prelude names may be shadowed by declarations in a module.

The preludes are layered such that one shadows another if they contain entities of the same name.
The order that preludes may shadow other preludes is the following where earlier entries may shadow later ones:

1. [Extern prelude]
2. [Tool prelude]
3. [`macro_use` prelude]
4. [Standard library prelude]
5. [Language prelude]

## `macro_rules` scopes

The scope of `macro_rules` macros is described in the [Macros By Example] chapter.
The behavior depends on the use of the [`macro_use`] and [`macro_export`] attributes.

## Derive macro helper attributes

[Derive macro helper attributes] are in scope in the item where their corresponding [`derive` attribute] is specified.
The scope extends from just after the `derive` attribute to the end of the item. <!-- Note: Not strictly true, see https://github.com/rust-lang/rust/issues/79202, but this is the intention. -->
Helper attributes shadow other attributes of the same name in scope.

## `Self` scope

Although [`Self`] is a keyword with special meaning, it interacts with name resolution in a way similar to normal names.

The implicit `Self` type in the definition of a [struct], [enum], [union], [trait], or [implementation] is treated similarly to a [generic parameter](#generic-parameter-scopes), and is in scope in the same way as a generic type parameter.

The implicit `Self` constructor in the value [namespace] of an [implementation] is in scope within the body of the implementation (the implementation's [associated items]).

```rust
// Self type within struct definition.
struct Recursive {
    f1: Option<Box<Self>>
}

// Self type within generic parameters.
struct SelfGeneric<T: Into<Self>>(T);

// Self value constructor within an implementation.
struct ImplExample();
impl ImplExample {
    fn example() -> Self { // Self type
        Self() // Self value constructor
    }
}
```

[_BareFunctionType_]: ../types/function-pointer.md
[_GenericParams_]: ../items/generics.md
[_TraitBound_]: ../trait-bounds.md
[_TypeBoundWhereClauseItem_]: ../items/generics.md
[`derive` attribute]: ../attributes/derive.md
[`for` loop]: ../expressions/loop-expr.md#iterator-loops
[`for`]: ../expressions/loop-expr.md#iterator-loops
[`if let`]: ../expressions/if-expr.md#if-let-expressions
[`let` statement]: ../statements.md#let-statements
[`macro_export`]: ../macros-by-example.md#path-based-scope
[`macro_use` prelude]: preludes.md#macro_use-prelude
[`macro_use`]: ../macros-by-example.md#the-macro_use-attribute
[`match` arms]: ../expressions/match-expr.md
[`Self`]: ../paths.md#self-1
[`while let`]: ../expressions/loop-expr.md#predicate-pattern-loops
[Associated consts]: ../items/associated-items.md#associated-constants
[associated items]: ../items/associated-items.md
[Asterisk glob imports]: ../items/use-declarations.md
[async blocks]: ../expressions/block-expr.md#async-blocks
[call expressions]: ../expressions/call-expr.md
[Closure parameter]: ../expressions/closure-expr.md
[closures]: ../expressions/closure-expr.md
[const arguments]: ../items/generics.md#const-generics
[const contexts]: ../const_eval.md#const-context
[Const generic parameters]: ../items/generics.md#const-generics
[Const items]: ../items/constant-items.md
[Constant]: ../items/constant-items.md
[Derive macro helper attributes]: ../procedural-macros.md#derive-macro-helper-attributes
[destructors]: ../destructors.md
[entity]: ../names.md
[enum]: ../items/enumerations.mdr
[enums]: ../items/enumerations.md
[Extern prelude]: preludes.md#extern-prelude
[Function parameter]: ../items/functions.md#function-parameters
[hrtb]: ../trait-bounds.md#higher-ranked-trait-bounds
[Impl trait]: ../types/impl-trait.md
[implementation]: ../items/implementations.md
[items]: ../items.md
[Language prelude]: preludes.md#language-prelude
[loop expression]: ../expressions/loop-expr.md
[Loop labels]: ../expressions/loop-expr.md#loop-labels
[Macros By Example]: ../macros-by-example.md
[match guard]: ../expressions/match-expr.md#match-guards
[methods]: ../items/associated-items.md#methods
[module]: ../items/modules.md
[name resolution]: name-resolution.md
[namespace]: namespaces.md
[path]: ../paths.md
[pattern]: ../patterns.md
[placeholder lifetime]: ../lifetime-elision.md
[preludes]: preludes.md
[Standard library prelude]: preludes.md#standard-library-prelude
[statement]: ../statements.md
[Static items]: ../items/static-items.md
[static]: ../items/static-items.md
[struct]: ../items/structs.md
[structs]: ../items/structs.md
[Tool prelude]: preludes.md#tool-prelude
[trait]: ../items/traits.md
[union]: ../items/unions.md
