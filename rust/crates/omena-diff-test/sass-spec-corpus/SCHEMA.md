# sass-spec seed corpus schema

This directory mirrors the WPT seed-corpus envelope used by `wpt-corpus/`:
`manifest.json` lists pinned upstream provenance, chunk manifests, sparse-path
fixture counts, and a peer known-failure policy.

Fixture chunks use the same top-level `schemaVersion`, `product`, `chunkId`,
`sourcePin`, and `fixtures` fields. Individual fixtures add dialect-specific
fields used by the Rust conformance tests:

- `dialect`: parser dialect (`scss` or `sass`).
- `source`: source text parsed by `omena-parser`.
- `expectedBogusKinds`: sorted expected `Bogus*` node kinds.
- `expectedErrorCodes`: sorted expected parser error codes.

The hard Rust gate asserts every fixture parses to a complete byte-covering CST.
The recorded-set gate asserts the actual bogus/error sets match these fixture
records and that every non-clean fixture is registered in the peer policy.
