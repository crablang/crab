# Implementing Arc

In this section, we'll be implementing a simpler version of `std::sync::Arc`.
Similarly to [the implementation of `Vec` we made earlier](../vec/vec.md), we won't be
taking advantage of as many optimizations, intrinsics, or unstable code as the
standard library may.

This implementation is loosely based on the standard library's implementation
(technically taken from `alloc::sync` in 1.49, as that's where it's actually
implemented), but it will not support weak references at the moment as they
make the implementation slightly more complex.

Please note that this section is very work-in-progress at the moment.
