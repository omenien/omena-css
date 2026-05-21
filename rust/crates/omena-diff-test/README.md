# `omena-diff-test`

`omena-diff-test` owns the Rust-side differential corpus harness for the
post-v5 `omena-css` track. It compares parser-owned facts from `omena-parser`
against legacy `engine-style-parser` oracle output while the legacy parser is
kept only as a baseline.

The crate is part of the parser cutover readiness path. Product consumers
should continue to depend on `omena-query` rather than this harness.

It also carries the M3 `cme-fixture-v0` seed corpus for future
`omena-testkit` promotion. Those seeds are intentionally small: they preserve
Sass-language, cascade-proof, provenance, and abstract-value cases without
turning M3 into the full M4 testkit migration.

The WPT seed lane starts as a Stage 1 advisory corpus. Once the fixture-count,
known-failure, and consecutive-green prerequisites are met, the checked-in
manifest records `stage2-blocking` and the known-failure policy records
`stage = "blocking"` plus `stage2_blocking = true`. This keeps the active gate
state visible in generated metadata rather than hiding it behind a boolean.
