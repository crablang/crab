Thank you for your interest in contributing to the Rust Reference!

There are a few ways of helping with the reference: critiquing the reference,
editing the reference, fixing incorrect information, adding examples and
glossary entries, and documenting new or otherwise undocumented features in
Rust.

For a while, the Reference was basically ignored, and Rust continued gaining new
features or changing old ones. It was also basically the introduction document
before the first edition of the Rust book, and constantly in flux from the huge
churn of the language design before v1.0.0. So there's a lot that's wrong, too
teachy for people who should have basic understanding of Rust, or is too shallow
for the Reference. As such, we have the warning saying there's work that needs
to be done. Eventually, we plan to make sure everything is well documented
enough that we can remove the warning.

It is encouraged for you to read the [introduction] to familiarize yourself
with the kind of content the reference is expected to contain and the
conventions it uses. Also, the [style guide] provides more detailed guidelines
for formatting and content.

## Critiquing the Reference

This is the easiest way to contribute. Basically, as you read the reference, if
you find something confusing, incorrect, or missing, then you can file an issue
against the reference explaining your concerns.

## Editing the Reference

Typos and incorrect links get through from time to time. Should you find them,
we welcome PRs to fix them. Additionally, larger editing jobs that help remove
the number of parentheticals, remove comma splices, italicize term definitions
and other similar tasks are helpful.

## Adding Examples and Glossary Entries

Examples are great. Many people will only read examples and ignore the prose.
Ideally, every facet of every feature will have an example.

Likewise, the reference has a glossary. It doesn't need to explain every facet
of every feature nor contain every definition, but it does need to be expanded
upon. Ideally entries in the glossary link to the associated documentation.

## Adding Documentation

There are a lot of features that are not documented at all or are documented
poorly. This is the hardest, but definitely most valuable. Pick an unassigned
issue from the [issue tracker], and write about it.

While writing, you may find it handy to have a [playpen] open to test out what
you are documenting.

Feel free to take information from the standard library and Rustonomicon as
appropriate.

Note that we don't write documentation for purely library features such as
threads and IO and we don't write about Rust in the future. Documentation is
written as if the current stable release of Rust is the last release. The
`master` branch of the reference corresponds to what is **stable** on the
`master` branch ("nightly") of [rust-lang/rust]. If you want to write about
Rust in the future, you want [the Unstable book][unstable].

## Stabilization

When something that alters the language is stabilized, an issue should be
opened on the reference [issue tracker] to track the documentation process.
This should include links to any relevant information, such as the
stabilization PR, the RFC, the tracking issue, and anything else that would be
helpful for writing the documentation.

[introduction]: src/introduction.md
[issue tracker]: https://github.com/rust-lang/reference/issues
[playpen]: https://play.rust-lang.org/
[rust-lang/rust]: https://github.com/rust-lang/rust/
[style guide]: STYLE.md
[unstable]: https://doc.rust-lang.org/nightly/unstable-book/
