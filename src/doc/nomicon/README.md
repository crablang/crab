# The Rustonomicon

The Dark Arts of Advanced and Unsafe Rust Programming

Nicknamed "the Nomicon."

## NOTE: This is a draft document, and may contain serious errors

> Instead of the programs I had hoped for, there came only a shuddering
blackness and ineffable loneliness; and I saw at last a fearful truth which no
one had ever dared to breathe before — the unwhisperable secret of secrets — The
fact that this language of stone and stridor is not a sentient perpetuation of
Rust as London is of Old London and Paris of Old Paris, but that it is in fact
quite unsafe, its sprawling body imperfectly embalmed and infested with queer
animate things which have nothing to do with it as it was in compilation.

This book digs into all the awful details that are necessary to understand in
order to write correct Unsafe Rust programs. Due to the nature of this problem,
it may lead to unleashing untold horrors that shatter your psyche into a billion
infinitesimal fragments of despair.

## Requirements

Building the Nomicon requires [mdBook]. To get it:

[mdBook]: https://github.com/rust-lang/mdBook

```bash
cargo install mdbook
```

### `mdbook` usage

To build the Nomicon use the `build` sub-command:

```bash
mdbook build
```

The output will be placed in the `book` subdirectory. To check it out, open the
`index.html` file in your web browser. You can pass the `--open` flag to `mdbook
build` and it'll open the index page in your default browser (if the process is
successful) just like with `cargo doc --open`:

```bash
mdbook build --open
```

There is also a `test` sub-command to test all code samples contained in the book:

```bash
mdbook test
```

### `linkcheck`

We use the `linkcheck` tool to find broken links.
To run it locally:

```sh
curl -sSLo linkcheck.sh https://raw.githubusercontent.com/rust-lang/rust/master/src/tools/linkchecker/linkcheck.sh
sh linkcheck.sh --all nomicon
```

## Contributing

Given that the Nomicon is still in a draft state, we'd love your help! Please
feel free to open issues about anything, and send in PRs for things you'd like
to fix or change. If your change is large, please open an issue first, so we can
make sure that it's something we'd accept before you go through the work of
getting a PR together.
