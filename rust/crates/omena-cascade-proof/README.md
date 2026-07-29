# omena-cascade-proof

## Role

`omena-cascade-proof` owns backend-neutral cascade proof inputs, verdicts,
discharge-ledger lookup, and telemetry used by diagnostics and transform
admission.

## Contract

It canonicalizes proof terms and exposes typed checks for shorthands, longhand
merges, scope and layer flattening, static `@supports`, and rewrite candidates.
The deterministic stub and optional solver backends return the same wire types.

## Consumers

Checker and transform paths consume these verdicts through product-owned
evidence contracts.

## Boundaries

The default backend is solver-free and must not be described as a formal proof.
Solver-backed experiments cannot change the public result shape.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-cascade-proof
```
