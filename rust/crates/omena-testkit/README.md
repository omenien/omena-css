# `omena-testkit`

`omena-testkit` owns shared Rust-side fixture primitives for omena-css tests.
The first surface is the `cme-fixture-v0` parser used by M4 conformance and
testkit migration work. Product crates can keep their own domain assertions,
but they should consume this shared fixture grammar instead of redefining local
fixture parsers.

This crate is intentionally small in M4: it locks the common fixture grammar,
promotion reporting path, the first scenario archetypes for boundary, transform
execution, LSP requests, and `shadow.omena(<verb>)` introspection, plus the seed
snapshot-governance policy for global-disable rejection, unreferenced snapshot
rejection, hot-snapshot age audit, and known-failure review policy.

Broader WPT corpus automation remains a separate Axis A surface.
