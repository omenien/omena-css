# `omena-diff-test`

`omena-diff-test` owns the Rust-side differential corpus harness for the
post-v5 `omena-css` track. It compares parser-owned facts from `omena-parser`
against legacy `engine-style-parser` oracle output while the legacy parser is
kept only as a baseline.

The crate is part of the parser cutover readiness path. Product consumers
should continue to depend on `omena-query` rather than this harness.

It also carries the M3 `omena-fixture-v0` seed corpus for future
`omena-testkit` promotion. Those seeds are intentionally small: they preserve
Sass-language, cascade-proof, provenance, and abstract-value cases without
turning M3 into the full M4 testkit migration.

The WPT seed lane keeps Stage 2 blocking fixtures and Stage 1 advisory fixtures
as separate generated chunks. The blocking chunk is tied to reviewed green-run
evidence in `known-failures/wpt-seed-policy.toml`; advisory chunks can grow the
corpus without invalidating that evidence. Once advisory fixtures have their own
review history, they can be promoted into the blocking chunk with updated
green-run evidence.

The Sass differential lane runs dart-sass 1.x as the Mode 1 compilability oracle
for a small SCSS corpus. When dart-sass compiles a fixture, the Omena diagnostic
path must not emit `missingSassSymbol`; any mismatch is classified before it can
silently regress the parser/query boundary.

The regression lane stores issue-linked `omena-fixture-v0` cases under
`regressions/{id}/`. Fixed fixtures must keep passing while linked issues are
closed; todo fixtures, when present, are allowed to fail only while their linked
issue remains open.
