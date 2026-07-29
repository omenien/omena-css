# omena-refinement-trait

## Role

`omena-refinement-trait` is the interface-only cycle breaker for refinement
predicates shared by cascade, refinement, and proof layers.

## Contract

It defines `PropertyIndexV0`, `RefinementPredicateV0`, typed verdicts,
provenance, witnesses, and refinement decisions without importing an
implementation backend.

## Consumers

Cascade, refinement, and proof crates exchange typed decisions through this
small dependency.

## Boundaries

The crate evaluates no CSS and runs no solver. Adding implementation
dependencies here would recreate the cycle it is designed to prevent.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-refinement-trait
```
