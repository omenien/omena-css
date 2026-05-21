# WPT Known-Failure Policy Schema

Known-failure policy files are TOML and scoped to a pinned corpus manifest.

Top-level fields:

- `schema_version`: schema version, currently `0`.
- `corpus_manifest`: relative path to the WPT seed manifest.
- `stage`: `advisory` or `blocking`.
- `stage2_blocking`: whether failures should block the Stage 2 conformance
  lane.
- `source_pin`: upstream WPT source pin.
- `review_interval_days`: maximum review interval for known-failure entries.
- `required_min_fixture_count_for_stage2`: minimum generated fixture count
  required before the seed corpus can be promoted to Stage 2 blocking.
- `required_consecutive_green_runs`: minimum consecutive green advisory runs
  required before Stage 2 blocking promotion.
- `consecutive_green_runs`: current reviewed consecutive green advisory run
  count for the pinned corpus and policy.

Future `[[subtest]]` entries must include `fixture`, `name`, `status`,
`reason`, `issue`, `since`, and `review_after`. Stale entries are rejected by
the checker when the fixture or subtest no longer exists.
