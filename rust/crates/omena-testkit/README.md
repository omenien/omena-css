# `omena-testkit`

`omena-testkit` owns shared Rust-side fixture primitives for omena-css tests.
The first surface is the `cme-fixture-v0` parser used by M4 conformance and
testkit migration work. Product crates can keep their own domain assertions,
but they should consume this shared fixture grammar instead of redefining local
fixture parsers.

This crate is intentionally small at M4 entry: it locks the common fixture
grammar and promotion reporting path before larger scenario macros, snapshot
governance, or WPT corpus automation are added.
