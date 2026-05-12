# `engine-input-producers`

Internal Rust crate for input-derived producer artifacts built from `EngineInputV2`.

This crate is the current Rust producer boundary inside the repo. It owns:

- family-level producer artifacts for `expression-semantics`, `source-resolution`, `expression-domain`, and `selector-usage`
- top-level lane artifacts for `source-side` and `semantic`
- canonical-candidate bundles, evaluator-candidate bundles, and canonical-producer signals where the input contract preserves enough evidence

Current lane calibration:

- unrestricted canonical lane:
  - `expression-semantics`
  - `source-resolution`
  - `source-side`
  - `semantic`
- bounded canonical lane:
  - `expression-domain`
  - evaluator coverage is intentionally limited to type-fact-backed corpora
- shadow-only lane:
  - `selector-usage`
  - current `EngineInputV2` does not preserve enough reference-level evidence to promote it beyond shadow validation

Primary checks:

- `cargo test --manifest-path rust/Cargo.toml -p engine-input-producers`
- `pnpm check:rust-source-side-lane`
- `pnpm check:rust-semantic-lane`
- `pnpm check:rust-producer-boundary`

Release-facing bundle:

- `pnpm check:rust-release-bundle`

This crate is intentionally internal. `publish = false` remains in effect at the workspace level.
