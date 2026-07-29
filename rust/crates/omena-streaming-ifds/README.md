# omena-streaming-ifds

## Role

`omena-streaming-ifds` provides incremental cross-file reachability contracts
for live LSP and CLI analysis.

## Contract

The default route is exact and wire-compatible with the shared hypergraph
model. Public reports describe updates, demand slices, fallbacks, settle
equivalence, readiness, and latency budgets.

## Consumers

CLI and LSP paths use its cross-file reachability reports over
`omena-cross-file-summary` identities.

## Boundaries

The optional dynamic-connectivity backend is an implementation boundary, not an
asymptotic proof claim. The crate does not parse source or publish diagnostics.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-streaming-ifds
```
