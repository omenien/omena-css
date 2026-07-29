# omena-query-transform-runner

## Role

`omena-query-transform-runner` groups planning, execution, printing, linking,
target selection, and plugin adapters behind one query dependency.

## Contract

It re-exports the transform-family contract and exposes built-in plugin
registration and execution through the same types consumed by `omena-query`.

## Consumers

`omena-query` uses this crate to keep its public build and transform APIs stable
without importing every transform implementation directly.

## Boundaries

The crate is an integration facade, not a second planner or printer. Ownership
of transform semantics remains in the underlying crates.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-query-transform-runner
```
