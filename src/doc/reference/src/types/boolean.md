# Boolean type

```rust
let b: bool = true;
```

The *boolean type* or *bool* is a primitive data type that can take on one of
two values, called *true* and *false*.

Values of this type may be created using a [literal expression] using the
keywords `true` and `false` corresponding to the value of the same name.

This type is a part of the [language prelude] with the [name] `bool`.

An object with the boolean type has a [size and alignment] of 1 each. The
value false has the bit pattern `0x00` and the value true has the bit pattern
`0x01`. It is [undefined behavior] for an object with the boolean type to have
any other bit pattern.

The boolean type is the type of many operands in various [expressions]:

* The condition operand in [if expressions] and [while expressions]
* The operands in [lazy boolean operator expressions][lazy]

> **Note**: The boolean type acts similarly to but is not an [enumerated type].
In practice, this mostly means that constructors are not associated to the type
(e.g. `bool::true`).

Like all primitives, the boolean type [implements][p-impl] the
[traits][p-traits] [`Clone`][p-clone], [`Copy`][p-copy], [`Sized`][p-sized],
[`Send`][p-send], and [`Sync`][p-sync].

> **Note**: See the [standard library docs][std] for library operations.

## Operations on boolean values

<!-- This is washy wording --> When using certain operator expressions with a
boolean type for its operands, they evaluate using the rules of [boolean logic].

### Logical not

| `b` | [`!b`][op-not] |
|- | - |
| `true` | `false` |
| `false` | `true` |

### Logical or

| `a` | `b` | [<code>a &#124; b</code>][op-or] |
|- | - | - |
| `true` | `true` | `true` |
| `true` | `false` | `true` |
| `false` | `true` | `true` |
| `false` | `false` | `false` |

### Logical and

| `a` | `b` | [`a & b`][op-and] |
|- | - | - |
| `true` | `true` | `true` |
| `true` | `false` | `false` |
| `false` | `true` | `false` |
| `false` | `false` | `false` |

### Logical xor

| `a` | `b` | [`a ^ b`][op-xor] |
|- | - | - |
| `true` | `true` | `false` |
| `true` | `false` | `true` |
| `false` | `true` | `true` |
| `false` | `false` | `false` |

### Comparisons

| `a` | `b` | [`a == b`][op-compare] |
|- | - | - |
| `true` | `true` | `true` |
| `true` | `false` | `false` |
| `false` | `true` | `false` |
| `false` | `false` | `true` |

| `a` | `b` | [`a > b`][op-compare] |
|- | - | - |
| `true` | `true` | `false` |
| `true` | `false` | `true` |
| `false` | `true` | `false` |
| `false` | `false` | `false` |

* `a != b` is the same as `!(a == b)`
* `a >= b` is the same as `a == b | a > b`
* `a < b` is the same as `!(a >= b)`
* `a <= b` is the same as `a == b | a < b`

## Bit validity

The single byte of a `bool` is guaranteed to be initialized (in other words,
`transmute::<bool, u8>(...)` is always sound -- but since some bit patterns
are invalid `bool`s, the inverse is not always sound).

[boolean logic]: https://en.wikipedia.org/wiki/Boolean_algebra
[enumerated type]: enum.md
[expressions]: ../expressions.md
[if expressions]: ../expressions/if-expr.md#if-expressions
[language prelude]: ../names/preludes.md#language-prelude
[lazy]: ../expressions/operator-expr.md#lazy-boolean-operators
[literal expression]: ../expressions/literal-expr.md
[name]: ../names.md
[op-and]: ../expressions/operator-expr.md#arithmetic-and-logical-binary-operators
[op-compare]: ../expressions/operator-expr.md#comparison-operators
[op-not]: ../expressions/operator-expr.md#negation-operators
[op-or]: ../expressions/operator-expr.md#arithmetic-and-logical-binary-operators
[op-xor]: ../expressions/operator-expr.md#arithmetic-and-logical-binary-operators
[p-clone]: ../special-types-and-traits.md#clone
[p-copy]: ../special-types-and-traits.md#copy
[p-impl]: ../items/implementations.md
[p-send]: ../special-types-and-traits.md#send
[p-sized]: ../special-types-and-traits.md#sized
[p-sync]: ../special-types-and-traits.md#sync
[p-traits]: ../items/traits.md
[size and alignment]: ../type-layout.md#size-and-alignment
[std]: ../../std/primitive.bool.html
[undefined behavior]: ../behavior-considered-undefined.md
[while expressions]: ../expressions/loop-expr.md#predicate-loops
