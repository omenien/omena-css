# omena-transform-cst

## Role

`omena-transform-cst` owns the transform pass taxonomy, ordering, build
profiles, editable transform IR, transaction validation, and rewrite evidence.

## Contract

Core APIs include `TransformPassKind`, `lower_transform_ir_from_source`,
`IrTransactionV0`, IR materialization, artifact construction, and
backend-parametric rewrite verification. Stable node keys and provenance keep
edits tied to parser-owned source regions.

## Consumers

Transform passes, printing, bundling, target planning, query transform routing,
and SCSS evaluation all depend on this contract.

## Boundaries

This crate does not implement every pass, print final CSS, or link modules.
Those responsibilities belong to `omena-transform-passes`,
`omena-transform-print`, and the bundler. The default proof path is a
deterministic stub, not an external theorem-prover guarantee.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-transform-cst
```
