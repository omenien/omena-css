# omena-query-checker-orchestrator

## Role

`omena-query-checker-orchestrator` is the checker handoff below the
consumer-facing `omena-query` facade.

## Contract

It invokes registered checker families, verifies emitted rule codes, and
returns evaluations with a typed diagnostic-gate summary.

## Consumers

`omena-query` uses this crate instead of depending on checker implementation
families individually.

## Boundaries

The crate does not own parser facts, query caching, severity rendering, or LSP
publication. It preserves the facade's public diagnostic contract.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-query-checker-orchestrator
```
