## Behavior not considered `unsafe`

The Rust compiler does not consider the following behaviors _unsafe_,
though a programmer may (should) find them undesirable, unexpected,
or erroneous.

##### Deadlocks
##### Leaks of memory and other resources
##### Exiting without calling destructors
##### Exposing randomized base addresses through pointer leaks
##### Integer overflow

If a program contains arithmetic overflow, the programmer has made an
error. In the following discussion, we maintain a distinction between
arithmetic overflow and wrapping arithmetic. The first is erroneous,
while the second is intentional.

When the programmer has enabled `debug_assert!` assertions (for
example, by enabling a non-optimized build), implementations must
insert dynamic checks that `panic` on overflow. Other kinds of builds
may result in `panics` or silently wrapped values on overflow, at the
implementation's discretion.

In the case of implicitly-wrapped overflow, implementations must
provide well-defined (even if still considered erroneous) results by
using two's complement overflow conventions.

The integral types provide inherent methods to allow programmers
explicitly to perform wrapping arithmetic. For example,
`i32::wrapping_add` provides two's complement, wrapping addition.

The standard library also provides a `Wrapping<T>` newtype which
ensures all standard arithmetic operations for `T` have wrapping
semantics.

See [RFC 560] for error conditions, rationale, and more details about
integer overflow.

##### Logic errors

Safe code may impose extra logical constraints that can be checked
at neither compile-time nor runtime. If a program breaks such
a constraint, the behavior may be unspecified but will not result in
undefined behavior. This could include panics, incorrect results,
aborts, and non-termination. The behavior may also differ between
runs, builds, or kinds of build.

For example, implementing both `Hash` and `Eq` requires that values
considered equal have equal hashes. Another example are data structures
like `BinaryHeap`, `BTreeMap`, `BTreeSet`, `HashMap` and `HashSet`
which describe constraints on the modification of their keys while
they are in the data structure. Violating such constraints is not
considered unsafe, yet the program is considered erroneous and
its behavior unpredictable.

[RFC 560]: https://github.com/rust-lang/rfcs/blob/master/text/0560-integer-overflow.md
