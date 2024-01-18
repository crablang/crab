# Comments

> **<sup>Lexer</sup>**\
> LINE_COMMENT :\
> &nbsp;&nbsp; &nbsp;&nbsp; `//` (~\[`/` `!` `\n`] | `//`) ~`\n`<sup>\*</sup>\
> &nbsp;&nbsp; | `//`
>
> BLOCK_COMMENT :\
> &nbsp;&nbsp; &nbsp;&nbsp; `/*` (~\[`*` `!`] | `**` | _BlockCommentOrDoc_)
>      (_BlockCommentOrDoc_ | ~`*/`)<sup>\*</sup> `*/`\
> &nbsp;&nbsp; | `/**/`\
> &nbsp;&nbsp; | `/***/`
>
> INNER_LINE_DOC :\
> &nbsp;&nbsp; `//!` ~\[`\n` _IsolatedCR_]<sup>\*</sup>
>
> INNER_BLOCK_DOC :\
> &nbsp;&nbsp; `/*!` ( _BlockCommentOrDoc_ | ~\[`*/` _IsolatedCR_] )<sup>\*</sup> `*/`
>
> OUTER_LINE_DOC :\
> &nbsp;&nbsp; `///` (~`/` ~\[`\n` _IsolatedCR_]<sup>\*</sup>)<sup>?</sup>
>
> OUTER_BLOCK_DOC :\
> &nbsp;&nbsp; `/**` (~`*` | _BlockCommentOrDoc_ )
>              (_BlockCommentOrDoc_ | ~\[`*/` _IsolatedCR_])<sup>\*</sup> `*/`
>
> _BlockCommentOrDoc_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; BLOCK_COMMENT\
> &nbsp;&nbsp; | OUTER_BLOCK_DOC\
> &nbsp;&nbsp; | INNER_BLOCK_DOC
>
> _IsolatedCR_ :\
> &nbsp;&nbsp; _A `\r` not followed by a `\n`_

## Non-doc comments

Comments follow the general C++ style of line (`//`) and
block (`/* ... */`) comment forms. Nested block comments are supported.

Non-doc comments are interpreted as a form of whitespace.

## Doc comments

Line doc comments beginning with exactly _three_ slashes (`///`), and block
doc comments (`/** ... */`), both outer doc comments, are interpreted as a
special syntax for [`doc` attributes]. That is, they are equivalent to writing
`#[doc="..."]` around the body of the comment, i.e., `/// Foo` turns into
`#[doc="Foo"]` and `/** Bar */` turns into `#[doc="Bar"]`.

Line comments beginning with `//!` and block comments `/*! ... */` are
doc comments that apply to the parent of the comment, rather than the item
that follows.  That is, they are equivalent to writing `#![doc="..."]` around
the body of the comment. `//!` comments are usually used to document
modules that occupy a source file.

Isolated CRs (`\r`), i.e. not followed by LF (`\n`), are not allowed in doc
comments.

## Examples

```rust
//! A doc comment that applies to the implicit anonymous module of this crate

pub mod outer_module {

    //!  - Inner line doc
    //!! - Still an inner line doc (but with a bang at the beginning)

    /*!  - Inner block doc */
    /*!! - Still an inner block doc (but with a bang at the beginning) */

    //   - Only a comment
    ///  - Outer line doc (exactly 3 slashes)
    //// - Only a comment

    /*   - Only a comment */
    /**  - Outer block doc (exactly) 2 asterisks */
    /*** - Only a comment */

    pub mod inner_module {}

    pub mod nested_comments {
        /* In Rust /* we can /* nest comments */ */ */

        // All three types of block comments can contain or be nested inside
        // any other type:

        /*   /* */  /** */  /*! */  */
        /*!  /* */  /** */  /*! */  */
        /**  /* */  /** */  /*! */  */
        pub mod dummy_item {}
    }

    pub mod degenerate_cases {
        // empty inner line doc
        //!

        // empty inner block doc
        /*!*/

        // empty line comment
        //

        // empty outer line doc
        ///

        // empty block comment
        /**/

        pub mod dummy_item {}

        // empty 2-asterisk block isn't a doc block, it is a block comment
        /***/

    }

    /* The next one isn't allowed because outer doc comments
       require an item that will receive the doc */

    /// Where is my item?
#   mod boo {}
}
```

[`doc` attributes]: ../rustdoc/the-doc-attribute.html
