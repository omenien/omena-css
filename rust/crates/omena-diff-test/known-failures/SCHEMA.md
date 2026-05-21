# WPT Known-Failure Policy Schema

Known-failure policy files are TOML and scoped to a pinned corpus manifest.

Top-level fields:

- `schema_version`: schema version, currently `0`.
- `corpus_manifest`: relative path to the WPT seed manifest.
- `stage`: `advisory` or `blocking`.
- `stage2_blocking`: whether failures should block the Stage 2 conformance
  lane.
- `source_pin`: upstream WPT source pin.

Future `[[subtest]]` entries must include `fixture`, `name`, `status`,
`reason`, `issue`, and `since`. Stale entries are rejected by the checker when
the fixture or subtest no longer exists.
