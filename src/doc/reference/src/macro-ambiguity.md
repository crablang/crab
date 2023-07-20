# Appendix: Macro Follow-Set Ambiguity Formal Specification

This page documents the formal specification of the follow rules for [Macros
By Example]. They were originally specified in [RFC 550], from which the bulk
of this text is copied, and expanded upon in subsequent RFCs.

## Definitions & Conventions

  - `macro`: anything invokable as `foo!(...)` in source code.
  - `MBE`: macro-by-example, a macro defined by `macro_rules`.
  - `matcher`: the left-hand-side of a rule in a `macro_rules` invocation, or a
    subportion thereof.
  - `macro parser`: the bit of code in the Rust parser that will parse the
    input using a grammar derived from all of the matchers.
  - `fragment`: The class of Rust syntax that a given matcher will accept (or
    "match").
  - `repetition` : a fragment that follows a regular repeating pattern
  - `NT`: non-terminal, the various "meta-variables" or repetition matchers
    that can appear in a matcher, specified in MBE syntax with a leading `$`
    character.
  - `simple NT`: a "meta-variable" non-terminal (further discussion below).
  - `complex NT`: a repetition matching non-terminal, specified via repetition
    operators (`*`, `+`, `?`).
  - `token`: an atomic element of a matcher; i.e. identifiers, operators,
    open/close delimiters, *and* simple NT's.
  - `token tree`: a tree structure formed from tokens (the leaves), complex
    NT's, and finite sequences of token trees.
  - `delimiter token`: a token that is meant to divide the end of one fragment
    and the start of the next fragment.
  - `separator token`: an optional delimiter token in an complex NT that
    separates each pair of elements in the matched repetition.
  - `separated complex NT`: a complex NT that has its own separator token.
  - `delimited sequence`: a sequence of token trees with appropriate open- and
    close-delimiters at the start and end of the sequence.
  - `empty fragment`: The class of invisible Rust syntax that separates tokens,
    i.e. whitespace, or (in some lexical contexts), the empty token sequence.
  - `fragment specifier`: The identifier in a simple NT that specifies which
    fragment the NT accepts.
  - `language`: a context-free language.

Example:

```rust,compile_fail
macro_rules! i_am_an_mbe {
    (start $foo:expr $($i:ident),* end) => ($foo)
}
```

`(start $foo:expr $($i:ident),* end)` is a matcher. The whole matcher is a
delimited sequence (with open- and close-delimiters `(` and `)`), and `$foo`
and `$i` are simple NT's with `expr` and `ident` as their respective fragment
specifiers.

`$(i:ident),*` is *also* an NT; it is a complex NT that matches a
comma-separated repetition of identifiers. The `,` is the separator token for
the complex NT; it occurs in between each pair of elements (if any) of the
matched fragment.

Another example of a complex NT is `$(hi $e:expr ;)+`, which matches any
fragment of the form `hi <expr>; hi <expr>; ...` where `hi <expr>;` occurs at
least once. Note that this complex NT does not have a dedicated separator
token.

(Note that Rust's parser ensures that delimited sequences always occur with
proper nesting of token tree structure and correct matching of open- and
close-delimiters.)

We will tend to use the variable "M" to stand for a matcher, variables "t" and
"u" for arbitrary individual tokens, and the variables "tt" and "uu" for
arbitrary token trees. (The use of "tt" does present potential ambiguity with
its additional role as a fragment specifier; but it will be clear from context
which interpretation is meant.)

"SEP" will range over separator tokens, "OP" over the repetition operators
`*`, `+`, and `?`, "OPEN"/"CLOSE" over matching token pairs surrounding a
delimited sequence (e.g. `[` and `]`).

Greek letters "α" "β" "γ" "δ"  stand for potentially empty token-tree sequences.
(However, the Greek letter "ε" (epsilon) has a special role in the presentation
and does not stand for a token-tree sequence.)

  * This Greek letter convention is usually just employed when the presence of
    a sequence is a technical detail; in particular, when we wish to *emphasize*
    that we are operating on a sequence of token-trees, we will use the notation
    "tt ..." for the sequence, not a Greek letter.

Note that a matcher is merely a token tree. A "simple NT", as mentioned above,
is an meta-variable NT; thus it is a non-repetition. For example, `$foo:ty` is
a simple NT but `$($foo:ty)+` is a complex NT.

Note also that in the context of this formalism, the term "token" generally
*includes* simple NTs.

Finally, it is useful for the reader to keep in mind that according to the
definitions of this formalism, no simple NT matches the empty fragment, and
likewise no token matches the empty fragment of Rust syntax. (Thus, the *only*
NT that can match the empty fragment is a complex NT.) This is not actually
true, because the `vis` matcher can match an empty fragment. Thus, for the
purposes of the formalism, we will treat `$v:vis` as actually being
`$($v:vis)?`, with a requirement that the matcher match an empty fragment.

### The Matcher Invariants

To be valid, a matcher must meet the following three invariants. The definitions
of FIRST and FOLLOW are described later.

1.  For any two successive token tree sequences in a matcher `M` (i.e. `M = ...
    tt uu ...`) with `uu ...` nonempty, we must have FOLLOW(`... tt`) ∪ {ε} ⊇
    FIRST(`uu ...`).
1.  For any separated complex NT in a matcher, `M = ... $(tt ...) SEP OP ...`,
    we must have `SEP` ∈ FOLLOW(`tt ...`).
1.  For an unseparated complex NT in a matcher, `M = ... $(tt ...) OP ...`, if
    OP = `*` or `+`, we must have FOLLOW(`tt ...`) ⊇ FIRST(`tt ...`).

The first invariant says that whatever actual token that comes after a matcher,
if any, must be somewhere in the predetermined follow set.  This ensures that a
legal macro definition will continue to assign the same determination as to
where `... tt` ends and `uu ...` begins, even as new syntactic forms are added
to the language.

The second invariant says that a separated complex NT must use a separator token
that is part of the predetermined follow set for the internal contents of the
NT. This ensures that a legal macro definition will continue to parse an input
fragment into the same delimited sequence of `tt ...`'s, even as new syntactic
forms are added to the language.

The third invariant says that when we have a complex NT that can match two or
more copies of the same thing with no separation in between, it must be
permissible for them to be placed next to each other as per the first invariant.
This invariant also requires they be nonempty, which eliminates a possible
ambiguity.

**NOTE: The third invariant is currently unenforced due to historical oversight
and significant reliance on the behaviour. It is currently undecided what to do
about this going forward. Macros that do not respect the behaviour may become
invalid in a future edition of Rust. See the [tracking issue].**

### FIRST and FOLLOW, informally

A given matcher M maps to three sets: FIRST(M), LAST(M) and FOLLOW(M).

Each of the three sets is made up of tokens. FIRST(M) and LAST(M) may also
contain a distinguished non-token element ε ("epsilon"), which indicates that M
can match the empty fragment. (But FOLLOW(M) is always just a set of tokens.)

Informally:

  * FIRST(M): collects the tokens potentially used first when matching a
    fragment to M.

  * LAST(M): collects the tokens potentially used last when matching a fragment
    to M.

  * FOLLOW(M): the set of tokens allowed to follow immediately after some
    fragment matched by M.

    In other words: t ∈ FOLLOW(M) if and only if there exists (potentially
    empty) token sequences α, β, γ, δ where:

      * M matches β,

      * t matches γ, and

      * The concatenation α β γ δ is a parseable Rust program.

We use the shorthand ANYTOKEN to denote the set of all tokens (including simple
NTs). For example, if any token is legal after a matcher M, then FOLLOW(M) =
ANYTOKEN.

(To review one's understanding of the above informal descriptions, the reader
at this point may want to jump ahead to the [examples of
FIRST/LAST](#examples-of-first-and-last) before reading their formal
definitions.)

### FIRST, LAST

Below are formal inductive definitions for FIRST and LAST.

"A ∪ B" denotes set union, "A ∩ B" denotes set intersection, and "A \ B"
denotes set difference (i.e. all elements of A that are not present in B).

#### FIRST

FIRST(M) is defined by case analysis on the sequence M and the structure of its
first token-tree (if any):

  * if M is the empty sequence, then FIRST(M) = { ε },

  * if M starts with a token t, then FIRST(M) = { t },

    (Note: this covers the case where M starts with a delimited token-tree
    sequence, `M = OPEN tt ... CLOSE ...`, in which case `t = OPEN` and thus
    FIRST(M) = { `OPEN` }.)

    (Note: this critically relies on the property that no simple NT matches the
    empty fragment.)

  * Otherwise, M is a token-tree sequence starting with a complex NT: `M = $( tt
    ... ) OP α`, or `M = $( tt ... ) SEP OP α`, (where `α` is the (potentially
    empty) sequence of token trees for the rest of the matcher).

      * Let SEP\_SET(M) = { SEP } if SEP is present and ε ∈ FIRST(`tt ...`);
        otherwise SEP\_SET(M) = {}.

  * Let ALPHA\_SET(M) = FIRST(`α`) if OP = `*` or `?` and ALPHA\_SET(M) = {} if
    OP = `+`.
  * FIRST(M) = (FIRST(`tt ...`) \\ {ε}) ∪ SEP\_SET(M) ∪ ALPHA\_SET(M).

The definition for complex NTs deserves some justification. SEP\_SET(M) defines
the possibility that the separator could be a valid first token for M, which
happens when there is a separator defined and the repeated fragment could be
empty. ALPHA\_SET(M) defines the possibility that the complex NT could be empty,
meaning that M's valid first tokens are those of the following token-tree
sequences `α`. This occurs when either `*` or `?` is used, in which case there
could be zero repetitions. In theory, this could also occur if `+` was used with
a potentially-empty repeating fragment, but this is forbidden by the third
invariant.

From there, clearly FIRST(M) can include any token from SEP\_SET(M) or
ALPHA\_SET(M), and if the complex NT match is nonempty, then any token starting
FIRST(`tt ...`) could work too. The last piece to consider is ε. SEP\_SET(M) and
FIRST(`tt ...`) \ {ε} cannot contain ε, but ALPHA\_SET(M) could. Hence, this
definition allows M to accept ε if and only if ε ∈ ALPHA\_SET(M) does. This is
correct because for M to accept ε in the complex NT case, both the complex NT
and α must accept it. If OP = `+`, meaning that the complex NT cannot be empty,
then by definition ε ∉ ALPHA\_SET(M). Otherwise, the complex NT can accept zero
repetitions, and then ALPHA\_SET(M) = FOLLOW(`α`). So this definition is correct
with respect to \varepsilon as well.

#### LAST

LAST(M), defined by case analysis on M itself (a sequence of token-trees):

  * if M is the empty sequence, then LAST(M) = { ε }

  * if M is a singleton token t, then LAST(M) = { t }

  * if M is the singleton complex NT repeating zero or more times, `M = $( tt
    ... ) *`, or `M = $( tt ... ) SEP *`

      * Let sep_set = { SEP } if SEP present; otherwise sep_set = {}.

      * if ε ∈ LAST(`tt ...`) then LAST(M) = LAST(`tt ...`) ∪ sep_set

      * otherwise, the sequence `tt ...` must be non-empty; LAST(M) = LAST(`tt
        ...`) ∪ {ε}.

  * if M is the singleton complex NT repeating one or more times, `M = $( tt ...
    ) +`, or `M = $( tt ... ) SEP +`

      * Let sep_set = { SEP } if SEP present; otherwise sep_set = {}.

      * if ε ∈ LAST(`tt ...`) then LAST(M) = LAST(`tt ...`) ∪ sep_set

      * otherwise, the sequence `tt ...` must be non-empty; LAST(M) = LAST(`tt
        ...`)

  * if M is the singleton complex NT repeating zero or one time, `M = $( tt ...)
    ?`, then LAST(M) = LAST(`tt ...`) ∪ {ε}.

  * if M is a delimited token-tree sequence `OPEN tt ... CLOSE`, then LAST(M) =
    { `CLOSE` }.

  * if M is a non-empty sequence of token-trees `tt uu ...`,

      * If ε ∈ LAST(`uu ...`), then LAST(M) = LAST(`tt`) ∪ (LAST(`uu ...`) \ { ε }).

      * Otherwise, the sequence `uu ...` must be non-empty; then LAST(M) =
        LAST(`uu ...`).

### Examples of FIRST and LAST

Below are some examples of FIRST and LAST.
(Note in particular how the special ε element is introduced and
eliminated based on the interaction between the pieces of the input.)

Our first example is presented in a tree structure to elaborate on how
the analysis of the matcher composes. (Some of the simpler subtrees
have been elided.)

```text
INPUT:  $(  $d:ident   $e:expr   );*    $( $( h )* );*    $( f ; )+   g
            ~~~~~~~~   ~~~~~~~                ~
                |         |                   |
FIRST:   { $d:ident }  { $e:expr }          { h }


INPUT:  $(  $d:ident   $e:expr   );*    $( $( h )* );*    $( f ; )+
            ~~~~~~~~~~~~~~~~~~             ~~~~~~~           ~~~
                        |                      |               |
FIRST:          { $d:ident }               { h, ε }         { f }

INPUT:  $(  $d:ident   $e:expr   );*    $( $( h )* );*    $( f ; )+   g
        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~    ~~~~~~~~~~~~~~    ~~~~~~~~~   ~
                        |                       |              |       |
FIRST:        { $d:ident, ε }            {  h, ε, ;  }      { f }   { g }


INPUT:  $(  $d:ident   $e:expr   );*    $( $( h )* );*    $( f ; )+   g
        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        |
FIRST:                       { $d:ident, h, ;,  f }
```

Thus:

 * FIRST(`$($d:ident $e:expr );* $( $(h)* );* $( f ;)+ g`) = { `$d:ident`, `h`, `;`, `f` }

Note however that:

 * FIRST(`$($d:ident $e:expr );* $( $(h)* );* $($( f ;)+ g)*`) = { `$d:ident`, `h`, `;`, `f`, ε }

Here are similar examples but now for LAST.

 * LAST(`$d:ident $e:expr`) = { `$e:expr` }
 * LAST(`$( $d:ident $e:expr );*`) = { `$e:expr`, ε }
 * LAST(`$( $d:ident $e:expr );* $(h)*`) = { `$e:expr`, ε, `h` }
 * LAST(`$( $d:ident $e:expr );* $(h)* $( f ;)+`) = { `;` }
 * LAST(`$( $d:ident $e:expr );* $(h)* $( f ;)+ g`) = { `g` }

### FOLLOW(M)

Finally, the definition for FOLLOW(M) is built up as follows. pat, expr, etc.
represent simple nonterminals with the given fragment specifier.

  * FOLLOW(pat) = {`=>`, `,`, `=`, `|`, `if`, `in`}`.

  * FOLLOW(expr) = FOLLOW(stmt) =  {`=>`, `,`, `;`}`.

  * FOLLOW(ty) = FOLLOW(path) = {`{`, `[`, `,`, `=>`, `:`, `=`, `>`, `>>`, `;`,
    `|`, `as`, `where`, block nonterminals}.

  * FOLLOW(vis) = {`,`l any keyword or identifier except a non-raw `priv`; any
    token that can begin a type; ident, ty, and path nonterminals}.

  * FOLLOW(t) = ANYTOKEN for any other simple token, including block, ident,
    tt, item, lifetime, literal and meta simple nonterminals, and all terminals.

  * FOLLOW(M), for any other M, is defined as the intersection, as t ranges over
    (LAST(M) \ {ε}), of FOLLOW(t).

The tokens that can begin a type are, as of this writing, {`(`, `[`, `!`, `*`,
`&`, `&&`, `?`, lifetimes, `>`, `>>`, `::`, any non-keyword identifier, `super`,
`self`, `Self`, `extern`, `crate`, `$crate`, `_`, `for`, `impl`, `fn`, `unsafe`,
`typeof`, `dyn`}, although this list may not be complete because people won't
always remember to update the appendix when new ones are added.

Examples of FOLLOW for complex M:

 * FOLLOW(`$( $d:ident $e:expr )*`) = FOLLOW(`$e:expr`)
 * FOLLOW(`$( $d:ident $e:expr )* $(;)*`) = FOLLOW(`$e:expr`) ∩ ANYTOKEN = FOLLOW(`$e:expr`)
 * FOLLOW(`$( $d:ident $e:expr )* $(;)* $( f |)+`) = ANYTOKEN

### Examples of valid and invalid matchers

With the above specification in hand, we can present arguments for
why particular matchers are legal and others are not.

 * `($ty:ty < foo ,)` : illegal, because FIRST(`< foo ,`) = { `<` } ⊈ FOLLOW(`ty`)

 * `($ty:ty , foo <)` : legal, because FIRST(`, foo <`) = { `,` }  is ⊆ FOLLOW(`ty`).

 * `($pa:pat $pb:pat $ty:ty ,)` : illegal, because FIRST(`$pb:pat $ty:ty ,`) = { `$pb:pat` } ⊈ FOLLOW(`pat`), and also FIRST(`$ty:ty ,`) = { `$ty:ty` } ⊈ FOLLOW(`pat`).

 * `( $($a:tt $b:tt)* ; )` : legal, because FIRST(`$b:tt`) = { `$b:tt` } is ⊆ FOLLOW(`tt`) = ANYTOKEN, as is FIRST(`;`) = { `;` }.

 * `( $($t:tt),* , $(t:tt),* )` : legal,  (though any attempt to actually use this macro will signal a local ambiguity error during expansion).

 * `($ty:ty $(; not sep)* -)` : illegal, because FIRST(`$(; not sep)* -`) = { `;`, `-` } is not in FOLLOW(`ty`).

 * `($($ty:ty)-+)` : illegal, because separator `-` is not in FOLLOW(`ty`).

 * `($($e:expr)*)` : illegal, because expr NTs are not in FOLLOW(expr NT).

[Macros by Example]: macros-by-example.md
[RFC 550]: https://github.com/rust-lang/rfcs/blob/master/text/0550-macro-future-proofing.md
[tracking issue]: https://github.com/rust-lang/rust/issues/56575
