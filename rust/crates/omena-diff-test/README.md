# `omena-diff-test`

`omena-diff-test` owns the Rust-side differential corpus harness for the
post-v5 `omena-css` track. It compares parser-owned facts from `omena-parser`
against legacy `engine-style-parser` oracle output while the legacy parser is
kept only as a baseline.

The crate is part of the parser cutover readiness path. Product consumers
should continue to depend on `omena-query` rather than this harness.
