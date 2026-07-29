# omena-query-core

## Role

`omena-query-core` contains producer-fragment summaries, expression-domain
runtime state, and abstract-value projection below the `omena-query` facade.

## Contract

The crate re-exports shared engine inputs and value types and provides the
low-level dataflow used to summarize selector usage, source resolution, class
values, and expression semantics.

## Consumers

`omena-query` composes these primitives into stable product queries.

## Boundaries

The crate does not own LSP or CLI routing, workspace persistence, diagnostic
rendering, or transform execution.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-query-core
```
