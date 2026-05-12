# omena-incremental

`omena-incremental` owns the small, serializable invalidation contract that lets
Omena runtimes decide which semantic nodes need recomputation between revisions.

Current public product:

- `omena-incremental.boundary` — boundary summary for the V0 dirty-node model.
- `plan_incremental_computation` — deterministic node invalidation over stable
  node IDs, input digests, and dependency edges.
- `snapshot_from_graph_input` — reusable snapshot materialization for callers
  that want to carry revision state across requests.
- `OmenaIncrementalDatabaseV0` — persistent Salsa-backed input store for the
  migration path from manual invalidation plans to tracked query reuse.
- `summarize_salsa_incremental_node_snapshot` — tracked node snapshot query that
  proves field-granular reuse before callers switch fully to the Salsa runtime.
- `IncrementalCancellationRegistryV0` — bounded cooperative cancellation state
  shared by Rust LSP and future query runtimes.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-incremental
```
