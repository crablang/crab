# Data Representation in Rust

Low-level programming cares a lot about data layout. It's a big deal. It also
pervasively influences the rest of the language, so we're going to start by
digging into how data is represented in Rust.

This chapter is ideally in agreement with, and rendered redundant by,
the [Type Layout section of the Reference][ref-type-layout]. When this
book was first written, the reference was in complete disrepair, and the
Rustonomicon was attempting to serve as a partial replacement for the reference.
This is no longer the case, so this whole chapter can ideally be deleted.

We'll keep this chapter around for a bit longer, but ideally you should be
contributing any new facts or improvements to the Reference instead.

[ref-type-layout]: ../reference/type-layout.html
