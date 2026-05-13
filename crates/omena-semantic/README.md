# `omena-semantic`

Internal Rust crate for the style semantic boundary.

This crate is intentionally small: it consumes parser-fact seeds from
`omena-parser`, owns the semantic contract DTOs consumed by bridge/query
layers, and exposes semantic-facing summaries without depending on the legacy
parser crate.

Current public products:

- `omena-semantic.style-semantic-graph` — combined parser boundary,
  CSS Modules semantic seeds, selector-identity, selector-reference,
  source-input evidence, promotion evidence, and lossless CST contract for
  semantic graph consumers.
- `omena-semantic.css-modules-semantics` — parser-fact-backed per-file CSS
  Modules seed summary for local class exports, `composes`, `@value`, ICSS,
  keyframes, and animation references. Cross-file resolution and closure
  analysis remain explicit next-priority gaps.
- `omena-semantic.selector-identity` — canonical selector ids, BEM suffix
  identity, and rewrite-safety blockers.
- `omena-semantic.selector-references` — selector-scoped reference summaries
  and identity-preserving reference sites derived from `EngineInputV2`.
- `omena-semantic.design-token-semantics` — CSS custom property resolver
  readiness surface that exposes same-file and occurrence-level resolution
  counts, source-order and layer-order facts, same-file cascade ranking records,
  selector/wrapper context signals, and the remaining cross-file/package/theme
  gaps.
- `omena-semantic.style-context-index` — explicit `@layer`, `@container`, and
  `@scope` indexes plus selector membership edges for cascade-aware semantic
  consumers.
- `omena-semantic.promotion-evidence` — explicit readiness/gap checklist for
  promotion beyond output parity, including parser-backed design-token seed
  evidence from CSS custom properties.
- `omena-semantic.source-input-evidence` — `EngineInputV2`-backed reference
  site identity, binding origin, style module edge, value-domain explanation,
  value-domain derivation, and selector certainty reason evidence.
- `omena-semantic.lossless-cst-contract` — byte-span invariants used by precise
  rename, formatter, and recovery-oriented consumers.
- `omena-semantic.theory-observation-harness` — observation-only readiness
  summary for selector rewrite safety, source evidence explainability, semantic
  graph downstream readiness, and generic-vs-CME coupling boundaries.

Primary check:

- `cargo test --manifest-path rust/Cargo.toml -p omena-semantic`

CLI smoke:

```sh
printf '.button { &__icon {} }' \
  | cargo run --manifest-path rust/Cargo.toml -p omena-semantic --bin omena-semantic-boundary -- Component.module.scss
```

For `EngineInputV2` source-side evidence:

```sh
cat engine-input-v2.json \
  | cargo run --manifest-path rust/Cargo.toml -p omena-semantic --bin omena-semantic-source-evidence
```

For the combined style semantic graph product:

```sh
cat style-semantic-graph-input.json \
  | cargo run --manifest-path rust/Cargo.toml -p omena-semantic --bin omena-semantic-graph
```

For the theory observation harness:

```sh
cat style-semantic-graph-input.json \
  | cargo run --manifest-path rust/Cargo.toml -p omena-semantic --bin omena-semantic-observation
```

For the compact observation contract:

```sh
cat style-semantic-graph-input.json \
  | cargo run --manifest-path rust/Cargo.toml -p omena-semantic --bin omena-semantic-observation-contract
```

Downstream consumers can use either the free function or the
`TheoryObservationHarnessInput` trait. The trait is the dogfooding surface for
consumers that should depend on an observation contract instead of a concrete
builder entry point. Consumers that only need stable readiness state can call
`summarize_theory_observation_contract` or the trait method
`summarize_theory_observation_contract`.
