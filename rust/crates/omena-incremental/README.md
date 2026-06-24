# omena-incremental

`omena-incremental` owns the small, serializable invalidation contract that lets
Omena runtimes decide which semantic nodes need recomputation between revisions.

Current public product:

- `omena-incremental.boundary` — boundary summary for the V0 dirty-node model.
- `snapshot_from_graph_input` — reusable snapshot materialization for callers
  that want to carry revision state across requests.
- `OmenaIncrementalDatabaseV0` — persistent Salsa-backed input store for the
  tracked-query reuse path, including seeded snapshots and graph updates.
- `summarize_salsa_incremental_node_snapshot` — tracked node snapshot query that
  proves field-granular reuse before callers switch fully to the Salsa runtime.
- `IncrementalCancellationRegistryV0` — bounded cooperative cancellation state
  shared by Rust LSP and future query runtimes.
- `summarize_incremental_layer_evidence_v0` — M6 evidence summary that limits
  incremental claims to stable-node invalidation, fuzzed dirty-set invariants,
  Salsa reuse, and Datalog-shaped audit contracts. It explicitly does not claim
  DBSP, Z-set, external Datalog execution, local timing superiority, or public
  safety.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-incremental
```
