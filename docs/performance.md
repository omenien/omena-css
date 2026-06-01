# Performance Baseline

This document defines the public benchmark surface for Omena CSS Modules.
It is intentionally reproducible rather than a one-off local timing note.

## Scope

The Z5 baseline covers two layers.

- Micro-benchmarks run directly against Rust crates with Criterion.
- Macro-benchmarks exercise the Rust LSP request path through JSON-RPC.

The current public corpus is synthetic but product-shaped:

- `nextjs14-dashboard-scss`: nested SCSS modules, `@use`, `@value`, custom properties, and BEM suffixes
- `vite-component-css`: many flat CSS module selectors plus keyframes
- `scss-heavy-design-system`: `@forward`, mixins, nested BEM selectors, and design-system-style variants

## Commands

Compile the benchmark harness without executing timing loops:

```bash
pnpm omena-check run rust/z5-performance-baseline-micro
```

Validate the benchmark boundary contract without running timing loops:

```bash
pnpm omena-check run rust/omena-benchmarks-boundary
```

Emit the machine-readable Criterion surface snapshot:

```bash
pnpm omena-check run rust/z5-criterion-surface-snapshot
```

Validate that the same Z5 style corpus is consumable by relevant external
comparators before making cross-tool benchmark claims:

```bash
pnpm omena-check run rust/z5-external-comparator-readiness
```

Run Criterion micro-benchmarks:

```bash
pnpm run benchmark:z5:micro
```

Run the LSP macro-benchmark:

```bash
pnpm run benchmark:z5:macro
```

Run the release-grade Z5 readiness check:

```bash
pnpm omena-check bundle rust/z5-performance-baseline-readiness
```

Run the parser-product cut-over ratio gate directly:

```bash
pnpm omena-check run rust/z5-parser-product-cutover
```

## Measured Surfaces

Criterion currently measures:

- `z5/parser`: legacy `engine-style-parser::parse_style_module` baseline kept for oracle comparison only
- `z5/omena-parser`: `omena-parser::parse` for the green-field parser track
- `z5/parser-product-legacy`: legacy CSS Modules intermediate producer
- `z5/parser-product-omena`: `omena-parser` CSS Modules intermediate producer
- `z5/semantic`: `omena-semantic::summarize_style_semantic_boundary`
- `z5/abstract-value`: 1-CFA flow analysis, one-CFA call-site batching, and reduced-product intersection

The LSP macro-benchmark measures:

- source hover
- source definition
- source completion
- style references
- event-loop probe latency while the above requests are in flight

## Comparison Policy

This repository does not claim a stable cross-tool speed ranking from local
machine timings alone. CMK, `typescript-plugin-css-modules`, and Biome CSS
comparisons should be published only when the compared workload and host
hardware are recorded with the same corpus, request mix, and cold/warm state.

Until then, the committed benchmark surface is the contract:

- benchmark code is versioned
- corpus generation is versioned
- macro request mix is versioned
- parser-product benchmark lanes expose a machine-readable readiness summary
  proving both lanes measure raw style source to product summary
- Criterion benchmark groups expose a machine-readable surface snapshot, so M4
  corpus expansion changes cannot silently miss a measured lane
- external comparator readiness proves `lightningcss` and `postcss` consume the
  same Z5 corpus snapshot for their supported dialects before any speed ranking
  is published
- parser-product cut-over ratio is enforced by `check:rust-z5-parser-product-cutover`
- thresholds are enforced by `check:rust-z5-performance-baseline-macro`

## Baseline Snapshot

The following numbers were captured on 2026-05-05 with:

- CPU: Apple M2 Max
- Memory: 32 GiB
- Command: `cargo bench --manifest-path rust/Cargo.toml -p omena-benchmarks --bench z5_performance_baseline -- --sample-size 10 --measurement-time 1 --warm-up-time 1`

| Surface        | Workload                       | Criterion interval |
| -------------- | ------------------------------ | ------------------ |
| parser         | `nextjs14-dashboard-scss`      | 81.601-83.774 us   |
| parser         | `vite-component-css`           | 142.06-142.83 us   |
| parser         | `scss-heavy-design-system`     | 60.004-60.287 us   |
| semantic       | `nextjs14-dashboard-scss`      | 5.0478-5.0847 ms   |
| semantic       | `vite-component-css`           | 2.6145-2.7621 ms   |
| semantic       | `scss-heavy-design-system`     | 1.8259-1.8387 ms   |
| abstract-value | `flow-1cfa-256-nodes`          | 170.95-173.33 us   |
| abstract-value | `one-cfa-40-call-sites`        | 1.9678-1.9803 ms   |
| abstract-value | `reduced-product-intersection` | 244.27-254.65 ns   |

`z5/omena-parser` was added after this timing snapshot. The current M4 gate now
records the Criterion surface structurally, but local timing numbers remain
engineering evidence rather than an external speed claim until a full refreshed
Criterion timing run is captured with host details.

## Parser-Product Cut-Over Snapshot

The parser-product cut-over gate compares the actual CSS Modules intermediate
producer path from raw style source to product summary, not full CST
construction alone. The legacy parser is retained here as a benchmark/oracle
baseline, not as a product parser lane dependency. The following numbers were
captured on 2026-05-19 with:

- Command: `pnpm omena-check run rust/z5-parser-product-cutover`
- Iterations: 40 per sample
- Max allowed ratio: `omena / legacy <= 1.10`

| Workload                   | omena-parser product | legacy product | Ratio |
| -------------------------- | -------------------: | -------------: | ----: |
| `nextjs14-dashboard-scss`  |             1.582 ms |       4.907 ms | 0.322 |
| `vite-component-css`       |             1.422 ms |       2.744 ms | 0.518 |
| `scss-heavy-design-system` |             0.955 ms |       1.820 ms | 0.525 |

The LSP macro-benchmark snapshot used:

```bash
OMENA_LSP_RUNTIME_LOOP_SELECTORS=24 \
OMENA_LSP_RUNTIME_LOOP_PROBE_DURATION_MS=400 \
OMENA_LSP_RUNTIME_LOOP_P95_MS=250 \
OMENA_LSP_RUNTIME_LOOP_MAX_MS=750 \
pnpm omena-check run rust/omena-lsp-server/runtime-loop
```

| Surface           | Samples |     p50 |     p95 |     max |
| ----------------- | ------: | ------: | ------: | ------: |
| source hover      |      24 | 3.51 ms | 5.72 ms | 5.87 ms |
| source definition |      12 | 2.92 ms | 5.79 ms | 5.79 ms |
| source completion |       5 | 3.43 ms | 5.56 ms | 5.56 ms |
| style references  |       5 | 3.15 ms | 5.27 ms | 5.27 ms |
| event-loop probe  |      19 |     n/a | 1.26 ms | 1.26 ms |
