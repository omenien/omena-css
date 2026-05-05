# Performance Baseline

This document defines the public benchmark surface for CSS Module Explainer.
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
pnpm cme-check run rust/z5-performance-baseline-micro
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
pnpm cme-check bundle rust/z5-performance-baseline-readiness
```

## Measured Surfaces

Criterion currently measures:

- `z5/parser`: `engine-style-parser::parse_style_module`
- `z5/omena-parser`: `omena-parser::parse` for the green-field parser track
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

`z5/omena-parser` was added after this snapshot. Recapture the snapshot before
publishing parser-track comparative claims.

The LSP macro-benchmark snapshot used:

```bash
CME_LSP_RUNTIME_LOOP_SELECTORS=24 \
CME_LSP_RUNTIME_LOOP_PROBE_DURATION_MS=400 \
CME_LSP_RUNTIME_LOOP_P95_MS=250 \
CME_LSP_RUNTIME_LOOP_MAX_MS=750 \
pnpm cme-check run rust/omena-lsp-server/runtime-loop
```

| Surface           | Samples |     p50 |     p95 |     max |
| ----------------- | ------: | ------: | ------: | ------: |
| source hover      |      24 | 3.27 ms | 5.45 ms | 5.60 ms |
| source definition |      12 | 2.71 ms | 5.51 ms | 5.51 ms |
| source completion |       5 | 3.20 ms | 5.29 ms | 5.29 ms |
| style references  |       5 | 2.94 ms | 5.01 ms | 5.01 ms |
| event-loop probe  |      19 |     n/a | 1.21 ms | 1.21 ms |
