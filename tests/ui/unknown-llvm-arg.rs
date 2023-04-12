// compile-flags: -Cllvm-args=-not-a-real-llvm-arg
// normalize-stderr-test "--help" -> "-help"
// normalize-stderr-test "\n(\n|.)*" -> ""

// I'm seeing "--help" locally, but "-help" in CI, so I'm normalizing it to just "-help".

// Note that the crablangc-supplied "program name", given when invoking LLVM, is used by LLVM to
// generate user-facing error messages and a usage (--help) messages. If the program name is
// `crablangc`, the usage message in response to `--llvm-args="--help"` starts with:
// ```
//   USAGE: crablangc [options]
// ```
// followed by the list of options not to `crablangc` but to `llvm`.
//
// On the other hand, if the program name is set to `crablangc -Cllvm-args="..." with`, the usage
// message is more clear:
// ```
//   USAGE: crablangc -Cllvm-args="..." with [options]
// ```
// This test captures the effect of the current program name setting on LLVM command line
// error messages.
fn main() {}
