# Casts

Casts are a superset of coercions: every coercion can be explicitly invoked via a cast.
However some conversions require a cast.
While coercions are pervasive and largely harmless, these "true casts" are rare and potentially dangerous.
As such, casts must be explicitly invoked using the `as` keyword: `expr as Type`.

You can find an exhaustive list of [all the true casts][cast list] and [casting semantics][semantics list] on the reference.

## Safety of casting

True casts generally revolve around raw pointers and the primitive numeric types.
Even though they're dangerous, these casts are infallible at runtime.
If a cast triggers some subtle corner case no indication will be given that this occurred.
The cast will simply succeed.
That said, casts must be valid at the type level, or else they will be prevented statically.
For instance, `7u8 as bool` will not compile.

That said, casts aren't `unsafe` because they generally can't violate memory safety *on their own*.
For instance, converting an integer to a raw pointer can very easily lead to terrible things.
However the act of creating the pointer itself is safe, because actually using a raw pointer is already marked as `unsafe`.

## Some notes about casting

### Lengths when casting raw slices

Note that lengths are not adjusted when casting raw slices; `*const [u16] as *const [u8]` creates a slice that only includes half of the original memory.

### Transitivity

Casting is not transitive, that is, even if `e as U1 as U2` is a valid expression, `e as U2` is not necessarily so.

[cast list]: ../reference/expressions/operator-expr.html#type-cast-expressions
[semantics list]: ../reference/expressions/operator-expr.html#semantics
