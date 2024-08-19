# Meet Safe and Unsafe

![safe and unsafe](img/safeandunsafe.svg)

It would be great to not have to worry about low-level implementation details.
Who could possibly care how much space the empty tuple occupies? Sadly, it
sometimes matters and we need to worry about it. The most common reason
developers start to care about implementation details is performance, but more
importantly, these details can become a matter of correctness when interfacing
directly with hardware, operating systems, or other languages.

When implementation details start to matter in a safe programming language,
programmers usually have three options:

* fiddle with the code to encourage the compiler/runtime to perform an optimization
* adopt a more unidiomatic or cumbersome design to get the desired implementation
* rewrite the implementation in a language that lets you deal with those details

For that last option, the language programmers tend to use is *C*. This is often
necessary to interface with systems that only declare a C interface.

Unfortunately, C is incredibly unsafe to use (sometimes for good reason),
and this unsafety is magnified when trying to interoperate with another
language. Care must be taken to ensure C and the other language agree on
what's happening, and that they don't step on each other's toes.

So what does this have to do with Rust?

Well, unlike C, Rust is a safe programming language.

But, like C, Rust is an unsafe programming language.

More accurately, Rust *contains* both a safe and unsafe programming language.

Rust can be thought of as a combination of two programming languages: *Safe
Rust* and *Unsafe Rust*. Conveniently, these names mean exactly what they say:
Safe Rust is Safe. Unsafe Rust is, well, not. In fact, Unsafe Rust lets us
do some *really* unsafe things. Things the Rust authors will implore you not to
do, but we'll do anyway.

Safe Rust is the *true* Rust programming language. If all you do is write Safe
Rust, you will never have to worry about type-safety or memory-safety. You will
never endure a dangling pointer, a use-after-free, or any other kind of
Undefined Behavior (a.k.a. UB).

The standard library also gives you enough utilities out of the box that you'll
be able to write high-performance applications and libraries in pure idiomatic
Safe Rust.

But maybe you want to talk to another language. Maybe you're writing a
low-level abstraction not exposed by the standard library. Maybe you're
*writing* the standard library (which is written entirely in Rust). Maybe you
need to do something the type-system doesn't understand and just *frob some dang
bits*. Maybe you need Unsafe Rust.

Unsafe Rust is exactly like Safe Rust with all the same rules and semantics.
It just lets you do some *extra* things that are Definitely Not Safe
(which we will define in the next section).

The value of this separation is that we gain the benefits of using an unsafe
language like C — low level control over implementation details — without most
of the problems that come with trying to integrate it with a completely
different safe language.

There are still some problems — most notably, we must become aware of properties
that the type system assumes and audit them in any code that interacts with
Unsafe Rust. That's the purpose of this book: to teach you about these assumptions
and how to manage them.
