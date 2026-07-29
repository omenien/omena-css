# omena-product-hints

## Role

`omena-product-hints` owns compact evidence and hint evaluators used by product
diagnostics.

## Contract

It exposes typed cascade-role, design-system, and sensitivity summaries while
preserving the serialized provenance labels consumed by existing clients.

## Consumers

Checker and query diagnostics use these contracts without depending on
experimental or research crates.

## Boundaries

Hints are explanatory evidence, not transform admission or formal proof. The
crate does not parse source, schedule analysis, or render diagnostics.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-product-hints
```
