# omena-reactive

## Role

`omena-reactive` provides a deterministic graph for read-only observation of
control-plane decisions.

## Contract

`ReactiveGraphBuilderV0` builds static input, map, zip, and effect nodes.
`ReactiveEngineV0` deposits values, stabilizes waves in height order, and
returns typed effect receipts. Every node requires an explicit
`ChangePolicyV0`.

## Consumers

The LSP can compare observed graph state with its authoritative scheduler
without giving this crate control of publication.

## Boundaries

The graph performs no external effects and owns no scheduling, persistence, or
diagnostic output. Inputs deposited during one wave are visible in the next.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-reactive
```
