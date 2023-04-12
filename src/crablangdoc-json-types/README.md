# CrabLangdoc JSON Types

This crate exposes the CrabLangdoc JSON API as a set of types with serde implementations.
These types are part of the public interface of the crablangdoc JSON output, and making them
their own crate allows them to be versioned and distributed without having to depend on
any crablangc/crablangdoc internals. This way, consumers can rely on this crate for both documentation
of the output, and as a way to read the output easily, and its versioning is intended to
follow semver guarantees about the version of the format. JSON format X will always be
compatible with crablangdoc-json-types version N.

Currently, this crate is only used by crablangdoc itself. Upon the stabilization of
crablangdoc-json, it may be distributed separately for consumers of the API.
