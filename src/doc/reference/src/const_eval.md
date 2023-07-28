# Constant evaluation

Constant evaluation is the process of computing the result of
[expressions] during compilation. Only a subset of all expressions
can be evaluated at compile-time.

## Constant expressions

Certain forms of expressions, called constant expressions, can be evaluated at
compile time. In [const contexts](#const-context), these are the only allowed
expressions, and are always evaluated at compile time. In other places, such as
[let statements], constant expressions *may*
be, but are not guaranteed to be, evaluated at compile time. Behaviors such as
out of bounds [array indexing] or [overflow] are compiler errors if the value
must be evaluated at compile time (i.e. in const contexts). Otherwise, these
behaviors are warnings, but will likely panic at run-time.

The following expressions are constant expressions, so long as any operands are
also constant expressions and do not cause any [`Drop::drop`][destructors] calls
to be run.

* [Literals].
* [Const parameters].
* [Paths] to [functions] and [constants].
  Recursively defining constants is not allowed.
* Paths to [statics]. These are only allowed within the initializer of a static.
* [Tuple expressions].
* [Array expressions].
* [Struct] expressions.
* [Block expressions], including `unsafe` blocks.
    * [let statements] and thus irrefutable [patterns], including mutable bindings
    * [assignment expressions]
    * [compound assignment expressions]
    * [expression statements]
* [Field] expressions.
* Index expressions, [array indexing] or [slice] with a `usize`.
* [Range expressions].
* [Closure expressions] which don't capture variables from the environment.
* Built-in [negation], [arithmetic], [logical], [comparison] or [lazy boolean]
  operators used on integer and floating point types, `bool`, and `char`.
* Shared [borrow]s, except if applied to a type with [interior mutability].
* The [dereference operator] except for raw pointers.
* [Grouped] expressions.
* [Cast] expressions, except
  * pointer to address casts and
  * function pointer to address casts.
* Calls of [const functions] and const methods.
* [loop], [while] and [`while let`] expressions.
* [if], [`if let`] and [match] expressions.

## Const context

A _const context_ is one of the following:

* [Array type length expressions]
* [Array repeat length expressions][array expressions]
* The initializer of
  * [constants]
  * [statics]
  * [enum discriminants]
* A [const generic argument]

## Const Functions

A _const fn_ is a function that one is permitted to call from a const context. Declaring a function
`const` has no effect on any existing uses, it only restricts the types that arguments and the
return type may use, as well as prevent various expressions from being used within it. You can freely
do anything with a const function that you can do with a regular function.

When called from a const context, the function is interpreted by the
compiler at compile time. The interpretation happens in the
environment of the compilation target and not the host. So `usize` is
`32` bits if you are compiling against a `32` bit system, irrelevant
of whether you are building on a `64` bit or a `32` bit system.

Const functions have various restrictions to make sure that they can be
evaluated at compile-time. It is, for example, not possible to write a random
number generator as a const function. Calling a const function at compile-time
will always yield the same result as calling it at runtime, even when called
multiple times. There's one exception to this rule: if you are doing complex
floating point operations in extreme situations, then you might get (very
slightly) different results. It is advisable to not make array lengths and enum
discriminants depend on floating point computations.


Notable features that are allowed in const contexts but not in const functions include:

* floating point operations
  * floating point values are treated just like generic parameters without trait bounds beyond
  `Copy`. So you cannot do anything with them but copy/move them around.

Conversely, the following are possible in a const function, but not in a const context:

* Use of generic type and lifetime parameters.
  * Const contexts do allow limited use of [const generic parameters].

[arithmetic]:           expressions/operator-expr.md#arithmetic-and-logical-binary-operators
[array expressions]:    expressions/array-expr.md
[array indexing]:       expressions/array-expr.md#array-and-slice-indexing-expressions
[array indexing]:       expressions/array-expr.md#array-and-slice-indexing-expressions
[array type length expressions]: types/array.md
[assignment expressions]: expressions/operator-expr.md#assignment-expressions
[compound assignment expressions]: expressions/operator-expr.md#compound-assignment-expressions
[block expressions]:    expressions/block-expr.md
[borrow]:               expressions/operator-expr.md#borrow-operators
[cast]:                 expressions/operator-expr.md#type-cast-expressions
[closure expressions]:  expressions/closure-expr.md
[comparison]:           expressions/operator-expr.md#comparison-operators
[const functions]:      items/functions.md#const-functions
[const generic argument]: items/generics.md#const-generics
[const generic parameters]: items/generics.md#const-generics
[constants]:            items/constant-items.md
[Const parameters]:     items/generics.md
[dereference operator]: expressions/operator-expr.md#the-dereference-operator
[destructors]:          destructors.md
[enum discriminants]:   items/enumerations.md#discriminants
[expression statements]: statements.md#expression-statements
[expressions]:          expressions.md
[field]:                expressions/field-expr.md
[functions]:            items/functions.md
[grouped]:              expressions/grouped-expr.md
[interior mutability]:  interior-mutability.md
[if]:                   expressions/if-expr.md#if-expressions
[`if let`]:             expressions/if-expr.md#if-let-expressions
[lazy boolean]:         expressions/operator-expr.md#lazy-boolean-operators
[let statements]:       statements.md#let-statements
[literals]:             expressions/literal-expr.md
[logical]:              expressions/operator-expr.md#arithmetic-and-logical-binary-operators
[loop]:                 expressions/loop-expr.md#infinite-loops
[match]:                expressions/match-expr.md
[negation]:             expressions/operator-expr.md#negation-operators
[overflow]:             expressions/operator-expr.md#overflow
[paths]:                expressions/path-expr.md
[patterns]:             patterns.md
[range expressions]:    expressions/range-expr.md
[slice]:                types/slice.md
[statics]:              items/static-items.md
[struct]:               expressions/struct-expr.md
[tuple expressions]:    expressions/tuple-expr.md
[while]:                expressions/loop-expr.md#predicate-loops
[`while let`]:          expressions/loop-expr.md#predicate-pattern-loops
