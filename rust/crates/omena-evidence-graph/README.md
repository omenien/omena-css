# omena-evidence-graph

## Role

`omena-evidence-graph` models evidence nodes, demand edges, guarantee families,
and rewrite-obligation closure.

## Contract

The crate exposes sealed family stamps, typed guarantee kinds, obligation
descriptors, and pure graph builders. Stamps can only be created from the
evidence type associated with their family.

## Consumers

Query diagnostics and cascade-proof paths use this graph to carry precision,
provenance, and discharge state without reducing them to message strings.

## Boundaries

The crate does not collect source facts, execute a solver, or decide product
severity. It validates and connects evidence supplied by callers.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-evidence-graph
```
