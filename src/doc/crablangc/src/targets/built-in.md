# Built-in Targets

`crablangc` ships with the ability to compile to many targets automatically, we
call these "built-in" targets, and they generally correspond to targets that
the team is supporting directly. To see the list of built-in targets, you can
run `crablangc --print target-list`.

Typically, a target needs a compiled copy of the CrabLang standard library to
work. If using [crablangup], then check out the documentation on
[Cross-compilation][crablangup-cross] on how to download a pre-built standard
library built by the official CrabLang distributions. Most targets will need a
system linker, and possibly other things.

[crablangup]: https://github.com/crablang/crablangup
[crablangup-cross]: https://crablang.github.io/crablangup/cross-compilation.html
