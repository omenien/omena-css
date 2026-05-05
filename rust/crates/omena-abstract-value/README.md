# `omena-abstract-value`

Internal Rust crate for the Omena abstract class-value domain.

This crate owns the generic domain contract used to reason about dynamic class
values before they are projected into a selector universe.

Current public product:

- `omena-abstract-value.domain` — domain boundary summary for the abstract
  class-value lattice and selector projection certainty contract.
- `intersect_abstract_class_values` — reduced-product intersection over finite,
  prefix, suffix, character-inclusion, and composite class-value domains.
- `join_abstract_class_values` — least-upper-bound merge for branch-sensitive
  class-value flow.
- `analyze_class_value_flow` — V0 1-CFA flow analysis over explicit
  class-value flow graphs with assign/refine/join transfers.
- `analyze_class_value_control_flow_graph` — CFG-aware flow analysis that
  prunes unreachable blocks before evaluating class-value transfer facts.
- `analyze_k_limited_call_site_flows` — k-limited call-site context analysis
  for dynamic class helpers that need deeper-than-1-CFA discrimination.
- `analyze_class_value_flow_incremental` — the same flow analysis surfaced with
  an `omena-incremental` recomputation plan and next snapshot.
- `analyze_class_value_flow_incremental_with_reuse` — incremental flow analysis
  that reuses the previous result when the plan is clean.
- `analyze_class_value_flow_incremental_with_database` — Salsa-backed
  incremental flow entrypoint that keeps `omena-incremental` DB state across
  requests while preserving the existing analysis result contract.
- `reduced_abstract_class_value_from_facts` /
  `reduced_value_domain_kind_from_facts` — source fact reduction before
  evaluator-facing domain-kind reporting.
- `summarize_abstract_class_value_provenance_tree` — self-contained V0
  explanation tree for exact, finite, constrained, widened, reduced-product,
  and flow-produced abstract class values.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-abstract-value
```

Split boundary check:

```sh
pnpm cme-check bundle rust/omena-abstract-value/split-boundary
```
