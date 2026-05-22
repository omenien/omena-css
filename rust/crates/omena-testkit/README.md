# `omena-testkit`

`omena-testkit` owns shared Rust-side fixture primitives for omena-css tests.
The first surface is the `cme-fixture-v0` parser used by M4 conformance and
testkit migration work. Product crates can keep their own domain assertions,
but they should consume this shared fixture grammar instead of redefining local
fixture parsers.

This crate is intentionally small in M4: it locks the common fixture grammar,
promotion reporting path, and the first scenario archetypes for boundary,
transform execution, LSP requests, and `shadow.omena(<verb>)` introspection.
Snapshot governance and broader WPT corpus automation remain separate Axis A
surfaces.
