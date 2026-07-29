# omena-cross-file-summary

## Role

`omena-cross-file-summary` owns the typed cross-file edge taxonomy, unified
hypergraph, SCC and closure algorithms, reverse-dependency index, and graph
deltas.

## Contract

The crate provides hypergraph and SCC summaries, reverse-dependency
construction and updates, diagnostics read sets, and IFDS summary-edge
tabulation over caller-supplied nodes and edges.

## Consumers

`omena-query` uses these summaries for cross-file diagnostics,
`omena-semantic` uses them for Sass and CSS Modules closure,
`omena-streaming-ifds` reuses the reachability model, and the bundler consumes
edge-order classifications.

## Boundaries

The crate does not parse source, resolve imports, canonicalize paths, perform
file I/O, or schedule live updates. Callers must supply normalized identities
and edge facts.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-cross-file-summary
```
