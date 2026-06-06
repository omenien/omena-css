# `omena-testkit`

`omena-testkit` owns shared Rust-side fixture primitives for omena-css tests.
The first surface is the `omena-fixture-v0` parser used by conformance and
testkit migration work. Product crates can keep their own domain assertions, but
they should consume this shared fixture grammar instead of redefining local
fixture parsers.

This crate locks the common fixture grammar, promotion reporting path, the first
scenario archetypes for boundary, transform execution, LSP requests, and
`shadow.omena(<verb>)` introspection, plus the seed snapshot-governance policy
for global-disable rejection, unreferenced snapshot rejection, hot-snapshot age
audit, known-failure review policy, and the first property-based SCSS parser
no-panic gate.

Broader WPT corpus automation remains a separate Axis A surface.
