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

The WPT seed lane keeps a Stage 1 advisory corpus shape while its
known-failure policy records the fixture-count and consecutive-green evidence
required for Stage 2. Once those prerequisites are met, the policy can enable
the Stage 2 blocking gate without hiding promotion blockers behind a boolean.
