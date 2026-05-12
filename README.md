# omena-css

Standalone Rust workspace for the Omena CSS parser, semantic substrates, cascade
model, and transform-planning crates.

This repository is staged from the CSS Module Explainer monorepo. The workspace
keeps the publish-target crates together so parser, incremental, cascade, and
transform boundaries can be verified as one product surface.

## Crates

- `omena-interner`
- `omena-syntax`
- `omena-parser`
- `omena-incremental`
- `omena-cascade`
- `omena-transform-cst`
- `omena-transform-passes`
- `omena-transform-bundle`
- `omena-transform-target`
- `omena-transform-print`
- `omena-transform-egg`

## Verification

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
