# Struct types

A `struct` *type* is a heterogeneous product of other types, called the
*fields* of the type.[^structtype]

New instances of a `struct` can be constructed with a [struct expression].

The memory layout of a `struct` is undefined by default to allow for compiler
optimizations like field reordering, but it can be fixed with the
[`repr` attribute]. In either case, fields may be given in any order in a
corresponding struct *expression*; the resulting `struct` value will always
have the same memory layout.

The fields of a `struct` may be qualified by [visibility modifiers], to allow
access to data in a struct outside a module.

A _tuple struct_ type is just like a struct type, except that the fields are
anonymous.

A _unit-like struct_ type is like a struct type, except that it has no fields.
The one value constructed by the associated [struct expression] is the only
value that inhabits such a type.

[^structtype]: `struct` types are analogous to `struct` types in C, the
    *record* types of the ML family, or the *struct* types of the Lisp family.

[`repr` attribute]: ../type-layout.md#representations
[struct expression]: ../expressions/struct-expr.md
[visibility modifiers]: ../visibility-and-privacy.md
