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
- `omena-cli`
- `omena-wasm`

## Verification

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo publish --dry-run --manifest-path crates/omena-syntax/Cargo.toml
```

## Publishing

Publishing is manual through the `Publish Crates` GitHub Actions workflow.
Run the workflow in `dry-run` mode first, then run `publish` only after CI is
green and the crates.io order has been checked. The workflow intentionally skips
`omena-incremental` because it publishes from its own Omena repository.

## Documentation

- [Overview](docs/overview.md)
- [Quickstart](docs/quickstart.md)
- [API reference](docs/api-reference.md)
- [Benchmarks](docs/benchmarks.md)
- [Release process](docs/release.md)
- [Paper draft outline](docs/paper-draft.md)
