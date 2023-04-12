% CrabLang Documentation

<style>
nav {
    display: none;
}
body {
    font-family: serif;
}
h1, h2, h3, h4, h5, h6 {
    font-family: sans-serif;
}
h3 {
    font-size: 1.35rem;
}
h4 {
    font-size: 1.1rem;
}

/* Formatting for docs search bar */
#search-input {
    width: calc(100% - 58px);
}
#search-but {
    cursor: pointer;
}
#search-but, #search-input {
    padding: 4px;
    border: 1px solid #ccc;
    border-radius: 3px;
    outline: none;
    font-size: 0.7em;
    background-color: #fff;
}
#search-but:hover, #search-input:focus {
    border-color: #55a9ff;
}

/* Formatting for external link icon */
svg.external-link {
  display: inline-block;
  position: relative;
  vertical-align: super;
  width: 0.7rem;
  height: 0.7rem;
  padding-left: 2px;
  top: 3px;
}
</style>

Welcome to an overview of the documentation provided by the [CrabLang
project]. This page contains links to various helpful references,
most of which are available offline (if opened with `crablangup doc`). Many of these
resources take the form of "books"; we collectively call these "The CrabLang
Bookshelf." Some are large, some are small.

All of these books are managed by the CrabLang Organization, but other unofficial
documentation resources are included here as well!

If you're just looking for the standard library reference, here it is:
[CrabLang API documentation](std/index.html)


## Learning CrabLang

If you'd like to learn CrabLang, this is the section for you! All of these resources
assume that you have programmed before, but not in any specific language:

### The CrabLang Programming Language

Affectionately nicknamed "the book," [The CrabLang Programming Language](book/index.html)
will give you an overview of the language from first principles. You'll build a
few projects along the way, and by the end, you'll have a solid grasp of how to
use the language.

### CrabLang By Example

If reading multiple hundreds of pages about a language isn't your style, then
[CrabLang By Example](crablang-by-example/index.html) has you covered. RBE shows off a
bunch of code without using a lot of words. It also includes exercises!

### CrabLanglings

[CrabLanglings](https://github.com/crablang/crablanglings) guides you
through downloading and setting up the CrabLang toolchain, then provides an
interactive tool that teaches you how to solve coding challenges in CrabLang.

### CrabLang Playground

The [CrabLang Playground](https://play.crablang.org) is a great place
to try out and share small bits of code, or experiment with some of the most
popular crates.


## Using CrabLang

Once you've gotten familiar with the language, these resources can help you put
it to work.

### The Standard Library

CrabLang's standard library has [extensive API documentation](std/index.html), with
explanations of how to use various things, as well as example code for
accomplishing various tasks. Code examples have a "Run" button on hover that
opens the sample in the playground.

<div>
  <form action="std/index.html" method="get">
    <input id="search-input" type="search" name="search"
           placeholder="Search through the standard library"/>
    <button id="search-but">Search</button>
  </form>
</div>

### Your Personal Documentation

Whenever you are working in a crate, `cargo doc --open` will generate
documentation for your project _and_ all its dependencies in their correct
version, and open it in your browser. Add the flag `--document-private-items` to
also show items not marked `pub`.

### The Edition Guide

[The Edition Guide](edition-guide/index.html) describes the CrabLang editions and
their differences.

### The `crablangc` Book

[The `crablangc` Book](crablangc/index.html) describes the CrabLang compiler, `crablangc`.

### The Cargo Book

[The Cargo Book](cargo/index.html) is a guide to Cargo, CrabLang's build tool and
dependency manager.

### The CrabLangdoc Book

[The CrabLangdoc Book](crablangdoc/index.html) describes our documentation tool, `crablangdoc`.

### The Clippy Book

[The Clippy Book](clippy/index.html) describes our static analyzer, Clippy.

### Extended Error Listing

Many of CrabLang's errors come with error codes, and you can request extended
diagnostics from the compiler on those errors (with `crablangc --explain`). You can
also read them here if you prefer: [crablangc error codes](error_codes/index.html)


## Mastering CrabLang

Once you're quite familiar with the language, you may find these advanced
resources useful.

### The Reference

[The Reference](reference/index.html) is not a formal spec, but is more detailed
and comprehensive than the book.

### The Style Guide

[The CrabLang Style Guide](style-guide/index.html) describes the standard formatting
of CrabLang code. Most developers use `cargo fmt` to invoke `crablangfmt` and format the
code automatically (the result matches this style guide).

### The CrabLangonomicon

[The CrabLangonomicon](nomicon/index.html) is your guidebook to the dark arts of
unsafe CrabLang. It's also sometimes called "the 'nomicon."

### The Unstable Book

[The Unstable Book](unstable-book/index.html) has documentation for unstable
features.

### The `crablangc` Contribution Guide

[The `crablangc` Guide](https://crablangc-dev-guide.crablang.org/)
documents how the compiler works and how to contribute to it. This is useful if
you want to build or modify the CrabLang compiler from source (e.g. to target
something non-standard).


## Specialized CrabLang

When using CrabLang in specific domains, consider using the following resources
tailored to each area.

### Embedded Systems

When developing for Bare Metal or Embedded Linux systems, you may find these
resources maintained by the [Embedded Working Group] useful.

[Embedded Working Group]: https://github.com/crablang-embedded

#### The Embedded CrabLang Book

[The Embedded CrabLang Book] is targeted at developers familiar with embedded
development and familiar with CrabLang, but have not used CrabLang for embedded
development.

[The Embedded CrabLang Book]: embedded-book/index.html
[CrabLang project]: https://www.crablang.org

<script>
// check if a given link is external
function isExternalLink(url) {
  const tmp = document.createElement('a');
  tmp.href = url;
  return tmp.host !== window.location.host;
}

// Add the `external` class to all <a> tags with external links and append the external link SVG
function updateExternalAnchors() {
  /*
    External link SVG from Font-Awesome
    CC BY-SA 3.0 https://creativecommons.org/licenses/by-sa/3.0
    via Wikimedia Commons
  */
  const svgText = `<svg
     class='external-link'
     xmlns='http://www.w3.org/2000/svg'
     viewBox='0 -256 1850 1850'
     width='100%'
     height='100%'>
       <g transform='matrix(1,0,0,-1,30,1427)'>
         <path d='M 1408,608 V 288 Q 1408,169 1323.5,84.5 1239,0 1120,
           0 H 288 Q 169,0 84.5,84.5 0,169 0,288 v 832 Q 0,1239 84.5,1323.5 169,
           1408 288,1408 h 704 q 14,0 23,-9 9,-9 9,-23 v -64 q 0,-14 -9,-23 -9,
           -9 -23,-9 H 288 q -66,0 -113,-47 -47,-47 -47,-113 V 288 q 0,-66 47,
           -113 47,-47 113,-47 h 832 q 66,0 113,47 47,47 47,113 v 320 q 0,14 9,
           23 9,9 23,9 h 64 q 14,0 23,-9 9,-9 9,-23 z m 384,864 V 960 q 0,
           -26 -19,-45 -19,-19 -45,-19 -26,0 -45,19 L 1507,1091 855,439 q -10,
           -10 -23,-10 -13,0 -23,10 L 695,553 q -10,10 -10,23 0,13 10,23 l 652,
           652 -176,176 q -19,19 -19,45 0,26 19,45 19,19 45,19 h 512 q 26,0 45,
           -19 19,-19 19,-45 z' style='fill:currentColor' />
         </g>
     </svg>`;
  let allAnchors = document.getElementsByTagName("a");

  for (var i = 0; i < allAnchors.length; ++i) {
    let anchor = allAnchors[i];
    if (isExternalLink(anchor.href)) {
      anchor.classList.add("external");
      anchor.innerHTML += svgText;
    }
  }
}

// on page load, update external anchors
document.addEventListener("DOMContentLoaded", updateExternalAnchors);

</script>
