# Enumerated types

An *enumerated type* is a nominal, heterogeneous disjoint union type, denoted
by the name of an [`enum` item]. [^enumtype]

An [`enum` item] declares both the type and a number of *variants*, each of
which is independently named and has the syntax of a struct, tuple struct or
unit-like struct.

New instances of an `enum` can be constructed with a [struct expression].

Any `enum` value consumes as much memory as the largest variant for its
corresponding `enum` type, as well as the size needed to store a discriminant.

Enum types cannot be denoted *structurally* as types, but must be denoted by
named reference to an [`enum` item].

[^enumtype]: The `enum` type is analogous to a `data` constructor declaration in
             ML, or a *pick ADT* in Limbo.

[`enum` item]: ../items/enumerations.md
[struct expression]: ../expressions/struct-expr.md
