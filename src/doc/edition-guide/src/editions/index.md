# What are Editions?

The release of Rust 1.0
([in May 2015](https://blog.rust-lang.org/2015/05/15/Rust-1.0.html))
established
["stability without stagnation"](https://blog.rust-lang.org/2014/10/30/Stability.html)
as a core Rust deliverable.
Ever since the 1.0 release,
the rule for Rust has been that once a feature has been released on stable,
we are committed to supporting that feature for all future releases.

There are times, however, when it is useful to be able to make small changes
to the language that are not backwards compatible.
The most obvious example is introducing a new keyword,
which would invalidate variables with the same name.
For example, the first version of Rust did not have the `async` and `await` keywords.
Suddenly changing those words to keywords in a later version would've broken code like `let async = 1;`.

**Editions** are the mechanism we use to solve this problem.
When we want to release a feature that would otherwise be backwards incompatible,
we do so as part of a new Rust *edition*.
Editions are opt-in, and so existing crates do
not see these changes until they explicitly migrate over to the new edition.
This means that even the latest version of Rust will still *not* treat `async` as a keyword,
unless edition 2018 or later is chosen.
This choice is made *per crate* [as part of its `Cargo.toml`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-edition-field).
New crates created by `cargo new` are always configured to use the latest stable edition.

### Editions do not split the ecosystem

The most important rule for editions is that crates in one edition can
interoperate seamlessly with crates compiled in other editions. This ensures
that the decision to migrate to a newer edition is a "private one" that the
crate can make without affecting others.

The requirement for crate interoperability implies some limits on the kinds of
changes that we can make in an edition.
In general, changes that occur in an edition tend to be "skin deep".
All Rust code, regardless of edition,
is ultimately compiled to the same internal representation within the compiler.

### Edition migration is easy and largely automated

Our goal is to make it easy for crates to upgrade to a new edition.
When we release a new edition,
we also provide [tooling to automate the migration](https://doc.rust-lang.org/cargo/commands/cargo-fix.html).
It makes minor changes to your code necessary to make it compatible with the new edition.
For example, when migrating to Rust 2018, it changes anything named `async` to use the equivalent
[raw identifier syntax](https://doc.rust-lang.org/rust-by-example/compatibility/raw_identifiers.html): `r#async`.

The automated migrations are not necessarily perfect:
there might be some corner cases where manual changes are still required.
The tooling tries hard to avoid changes
to semantics that could affect the correctness or performance of the code.

In addition to tooling, we also maintain this Rust Edition Guide that covers
the changes that are part of an edition.
This guide describes each change and gives pointers to where you can learn more about it.
It also covers any corner cases or details you should be aware of.
This guide serves as an overview of editions,
as a migration guide for specific editions,
and as a quick troubleshooting reference
if you encounter problems with the automated tooling.
