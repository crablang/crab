# `asm_unwind`

The tracking issue for this feature is: [#93334]

[#93334]: https://github.com/crablang/crablang/issues/93334

------------------------

This feature adds a `may_unwind` option to `asm!` which allows an `asm` block to unwind stack and be part of the stack unwinding process. This option is only supported by the LLVM backend right now.
