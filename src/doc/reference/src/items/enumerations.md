# Enumerations

> **<sup>Syntax</sup>**\
> _Enumeration_ :\
> &nbsp;&nbsp; `enum`
>    [IDENTIFIER]&nbsp;
>    [_GenericParams_]<sup>?</sup>
>    [_WhereClause_]<sup>?</sup>
>    `{` _EnumItems_<sup>?</sup> `}`
>
> _EnumItems_ :\
> &nbsp;&nbsp; _EnumItem_ ( `,` _EnumItem_ )<sup>\*</sup> `,`<sup>?</sup>
>
> _EnumItem_ :\
> &nbsp;&nbsp; _OuterAttribute_<sup>\*</sup> [_Visibility_]<sup>?</sup>\
> &nbsp;&nbsp; [IDENTIFIER]&nbsp;( _EnumItemTuple_ | _EnumItemStruct_ )<sup>?</sup>
>                                _EnumItemDiscriminant_<sup>?</sup>
>
> _EnumItemTuple_ :\
> &nbsp;&nbsp; `(` [_TupleFields_]<sup>?</sup> `)`
>
> _EnumItemStruct_ :\
> &nbsp;&nbsp; `{` [_StructFields_]<sup>?</sup> `}`
>
> _EnumItemDiscriminant_ :\
> &nbsp;&nbsp; `=` [_Expression_]

An *enumeration*, also referred to as an *enum*, is a simultaneous definition of a
nominal [enumerated type] as well as a set of *constructors*, that can be used
to create or pattern-match values of the corresponding enumerated type.

Enumerations are declared with the keyword `enum`.

An example of an `enum` item and its use:

```rust
enum Animal {
    Dog,
    Cat,
}

let mut a: Animal = Animal::Dog;
a = Animal::Cat;
```

Enum constructors can have either named or unnamed fields:

```rust
enum Animal {
    Dog(String, f64),
    Cat { name: String, weight: f64 },
}

let mut a: Animal = Animal::Dog("Cocoa".to_string(), 37.2);
a = Animal::Cat { name: "Spotty".to_string(), weight: 2.7 };
```

In this example, `Cat` is a _struct-like enum variant_, whereas `Dog` is simply
called an enum variant.

An enum where no constructors contain fields are called a
*<span id="field-less-enum">field-less enum</span>*. For example, this is a fieldless enum:

```rust
enum Fieldless {
    Tuple(),
    Struct{},
    Unit,
}
```

If a field-less enum only contains unit variants, the enum is called an
*<span id="unit-only-enum">unit-only enum</span>*. For example:

```rust
enum Enum {
    Foo = 3,
    Bar = 2,
    Baz = 1,
}
```

<span id="custom-discriminant-values-for-fieldless-enumerations"></span>
## Discriminants

Each enum instance has a _discriminant_: an integer logically associated to it
that is used to determine which variant it holds.

Under the [default representation], the discriminant is interpreted as
an `isize` value. However, the compiler is allowed to use a smaller type (or
another means of distinguishing variants) in its actual memory layout.

### Assigning discriminant values

#### Explicit discriminants

In two circumstances, the discriminant of a variant may be explicitly set by
following the variant name with `=` and a [constant expression]:


1. if the enumeration is "[unit-only]".


2. if a [primitive representation] is used. For example:

   ```rust
   #[repr(u8)]
   enum Enum {
       Unit = 3,
       Tuple(u16),
       Struct {
           a: u8,
           b: u16,
       } = 1,
   }
   ```

#### Implicit discriminants

If a discriminant for a variant is not specified, then it is set to one higher
than the discriminant of the previous variant in the declaration. If the
discriminant of the first variant in the declaration is unspecified, then
it is set to zero.

```rust
enum Foo {
    Bar,            // 0
    Baz = 123,      // 123
    Quux,           // 124
}

let baz_discriminant = Foo::Baz as u32;
assert_eq!(baz_discriminant, 123);
```

#### Restrictions

It is an error when two variants share the same discriminant.

```rust,compile_fail
enum SharedDiscriminantError {
    SharedA = 1,
    SharedB = 1
}

enum SharedDiscriminantError2 {
    Zero,       // 0
    One,        // 1
    OneToo = 1  // 1 (collision with previous!)
}
```

It is also an error to have an unspecified discriminant where the previous
discriminant is the maximum value for the size of the discriminant.

```rust,compile_fail
#[repr(u8)]
enum OverflowingDiscriminantError {
    Max = 255,
    MaxPlusOne // Would be 256, but that overflows the enum.
}

#[repr(u8)]
enum OverflowingDiscriminantError2 {
    MaxMinusOne = 254, // 254
    Max,               // 255
    MaxPlusOne         // Would be 256, but that overflows the enum.
}
```

### Accessing discriminant

#### Via `mem::discriminant`

[`mem::discriminant`] returns an opaque reference to the discriminant of
an enum value which can be compared. This cannot be used to get the value
of the discriminant.

#### Casting

If an enumeration is [unit-only] (with no tuple and struct variants), then its
discriminant can be directly accessed with a [numeric cast]; e.g.:

```rust
enum Enum {
    Foo,
    Bar,
    Baz,
}

assert_eq!(0, Enum::Foo as isize);
assert_eq!(1, Enum::Bar as isize);
assert_eq!(2, Enum::Baz as isize);
```

[Field-less enums] can be casted if they do not have explicit discriminants, or where only unit variants are explicit.

```rust
enum Fieldless {
    Tuple(),
    Struct{},
    Unit,
}

assert_eq!(0, Fieldless::Tuple() as isize);
assert_eq!(1, Fieldless::Struct{} as isize);
assert_eq!(2, Fieldless::Unit as isize);

#[repr(u8)]
enum FieldlessWithDiscrimants {
    First = 10,
    Tuple(),
    Second = 20,
    Struct{},
    Unit,
}

assert_eq!(10, FieldlessWithDiscrimants::First as u8);
assert_eq!(11, FieldlessWithDiscrimants::Tuple() as u8);
assert_eq!(20, FieldlessWithDiscrimants::Second as u8);
assert_eq!(21, FieldlessWithDiscrimants::Struct{} as u8);
assert_eq!(22, FieldlessWithDiscrimants::Unit as u8);
```

#### Pointer casting

If the enumeration specifies a [primitive representation], then the
discriminant may be reliably accessed via unsafe pointer casting:

```rust
#[repr(u8)]
enum Enum {
    Unit,
    Tuple(bool),
    Struct{a: bool},
}

impl Enum {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

let unit_like = Enum::Unit;
let tuple_like = Enum::Tuple(true);
let struct_like = Enum::Struct{a: false};

assert_eq!(0, unit_like.discriminant());
assert_eq!(1, tuple_like.discriminant());
assert_eq!(2, struct_like.discriminant());
```

## Zero-variant enums

Enums with zero variants are known as *zero-variant enums*. As they have
no valid values, they cannot be instantiated.

```rust
enum ZeroVariants {}
```

Zero-variant enums are equivalent to the [never type], but they cannot be
coerced into other types.

```rust,compile_fail
# enum ZeroVariants {}
let x: ZeroVariants = panic!();
let y: u32 = x; // mismatched type error
```

## Variant visibility

Enum variants syntactically allow a [_Visibility_] annotation, but this is
rejected when the enum is validated. This allows items to be parsed with a
unified syntax across different contexts where they are used.

```rust
macro_rules! mac_variant {
    ($vis:vis $name:ident) => {
        enum $name {
            $vis Unit,

            $vis Tuple(u8, u16),

            $vis Struct { f: u8 },
        }
    }
}

// Empty `vis` is allowed.
mac_variant! { E }

// This is allowed, since it is removed before being validated.
#[cfg(FALSE)]
enum E {
    pub U,
    pub(crate) T(u8),
    pub(super) T { f: String }
}
```

[IDENTIFIER]: ../identifiers.md
[_GenericParams_]: generics.md
[_WhereClause_]: generics.md#where-clauses
[_Expression_]: ../expressions.md
[_TupleFields_]: structs.md
[_StructFields_]: structs.md
[_Visibility_]: ../visibility-and-privacy.md
[enumerated type]: ../types/enum.md
[`mem::discriminant`]: ../../std/mem/fn.discriminant.html
[never type]: ../types/never.md
[unit-only]: #unit-only-enum
[numeric cast]: ../expressions/operator-expr.md#semantics
[constant expression]: ../const_eval.md#constant-expressions
[default representation]: ../type-layout.md#the-default-representation
[primitive representation]: ../type-layout.md#primitive-representations
[`C` representation]: ../type-layout.md#the-c-representation
[Field-less enums]: #field-less-enum
