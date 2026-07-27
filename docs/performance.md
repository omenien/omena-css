# Performance Evidence

This document is the reproducible entry point for omena-css performance work.
It explains what is measured, how to run it, and which committed artifact owns
published numbers. It does not turn a local timing into a product speed claim.

## Corpus

The current benchmark inventory contains 13 samples across two code-owned
corpus families:

- `style_corpus()` produces 10 product-shaped CSS and SCSS samples covering
  CSS Modules, nested Sass, layout, typography, backgrounds, transforms, and
  box-model workloads.
- `bundler_productization_corpus()` contributes three build fixtures: vendored
  Bootstrap Reboot 5.3.3, a Next.js-style SCSS module, and a CSS Modules product
  grid.

The inventory is defined in
`rust/crates/omena-benchmarks/src/corpus.rs`. Corpus membership, generated
source, emitted CSS, and comparator eligibility are versioned inputs rather
than unrecorded local fixtures.

## Reproduction Commands

Compile the Criterion harness without executing timing loops:

```bash
pnpm omena-check run rust/z5-performance-baseline-micro
```

Validate the benchmark boundary contract:

```bash
pnpm omena-check run rust/omena-benchmarks-boundary
```

Validate emitted CSS bytes against the committed omena golden snapshot:

```bash
pnpm omena-check run rust/benchmark/emitted-css-golden-gate
```

Measure the headline-axis fidelity snapshot without publishing a speed claim:

```bash
pnpm omena-check run rust/benchmark/headline-axis
```

Emit the machine-readable Criterion surface snapshot:

```bash
pnpm omena-check run rust/z5-criterion-surface-snapshot
```

Verify that supported external comparators can consume the same corpus before
making a cross-tool claim:

```bash
pnpm omena-check run rust/z5-external-comparator-readiness
```

Run the Criterion micro-benchmarks:

```bash
pnpm run benchmark:z5:micro
```

Run the LSP macro-benchmark:

```bash
pnpm run benchmark:z5:macro
```

Run the release-grade readiness bundle:

```bash
pnpm omena-check bundle rust/z5-performance-baseline-readiness
```

Run the advisory instruction-count probe:

```bash
pnpm omena-check run rust/benchmark/instruction-count-advisory
```

This probe compiles everywhere but records real instruction counts only where
Valgrind is available. It remains a scheduled/manual advisory lane rather than
a pull-request blocker until the relevant host and runtime costs are recorded.

Run the parser-product cut-over ratio gate directly:

```bash
pnpm omena-check run rust/z5-parser-product-cutover
```

Verify the committed instruction-count baseline and its metadata:

```bash
pnpm check:rust-z5-perf-baseline
```

The LSP runtime-loop configuration used by the macro gate is reproducible as:

```bash
OMENA_LSP_RUNTIME_LOOP_SELECTORS=24 \
OMENA_LSP_RUNTIME_LOOP_PROBE_DURATION_MS=400 \
OMENA_LSP_RUNTIME_LOOP_P95_MS=250 \
OMENA_LSP_RUNTIME_LOOP_MAX_MS=750 \
pnpm omena-check run rust/omena-lsp-server/runtime-loop
```

## Measured Surfaces

Criterion currently measures six product boundaries:

- `z5/parser`: the legacy `engine-style-parser::parse_style_module` oracle lane.
- `z5/omena-parser`: `omena-parser::parse` on the native parser path.
- `z5/parser-product-legacy`: the legacy CSS Modules intermediate producer.
- `z5/parser-product-omena`: the native CSS Modules intermediate producer.
- `z5/semantic`: `omena-semantic::summarize_style_semantic_boundary`.
- `z5/abstract-value`: 1-CFA flow analysis, call-site batching, and reduced-product intersection.

The LSP macro-benchmark measures source hover, source definition, source
completion, style references, and event-loop probe latency while requests are
in flight.

The committed instruction-count artifact additionally records cold open,
memoized recheck, committed-graph edit rebuild, and property-metadata lookup at
explicit corpus scales. Some comparisons enforce a slope; others are recorded
as baselines until their policy is strong enough to become a blocking contract.

## Evidence And Publication Boundary

This section is the normative publication policy. Benchmark changes
must report the command, input set, machine class, and comparison baseline.

- Publish only with the exact command, input set, machine class, Git SHA, and
  comparison boundary.
- Compare tools only on the same corpus, host, request mix, and cold/warm state.
- Validate emitted CSS and provenance before treating timing as comparable.
- Treat comparator-provided numbers as unverified until reproduced locally.
- Disclose slower and mixed outcomes under the same evidence standard.
- A green gate proves the stated contract and artifact, not that omena-css is
  the fastest implementation.

`speed_claim_ready` remains false on current benchmark snapshots. Fidelity
evidence covers Source Map V3 decoding, decoded segment validity, CSS Modules
`composes`/`:global` preservation, and provenance overhead. It does not by
itself authorize a competitive speed claim.

### Surface Disposition

The current benchmark surface is retained rather than deleted:

- Criterion Z5 benchmarks remain the micro-benchmark surface and readiness
  compile gate.
- Parser product cut-over remains an internal parity guardrail against Omena's
  own legacy parser lane.
- Bundler productization scripts remain manual profiling tools until a
  schema-versioned artifact renderer consumes them.
- Iai-Callgrind instruction-count coverage is advisory until Valgrind
  compatibility and runtime cost are recorded on scheduled Linux runs.
- Headline-axis fidelity coverage is a measurement gate only. It checks
  source-map decodeability, provenance overhead, and CSS Modules preservation;
  it does not publish a cross-tool speed claim.
- Fuzz checks cover parser, cascade, incremental, and transform safety targets.
  They are safety coverage, not benchmark surfaces.

### Reporting Template

```text
Command:
Inputs:
Machine:
Baseline:
Result:
Regression threshold:
Notes:
```

### Artifact Schema

Machine-readable benchmark artifacts use `schemaVersion: "0"` and must include:

- `generatedAtUtc`, `omenaGitSha`, `machine`, `container`, and `toolchain`
- `corpus[]` entries with name, path, SHA-256, byte length, line count, dialect, and provenance source
- `runner` with command, tool, iterations, warmup, and measured operation
- `results[]` with lane, metric, value, unit, variance or confidence interval, and run count
- `comparison` and `vendorReportedFlags`

Artifacts are build outputs, not a growing git-tracked benchmark history. A
benchmark number without the artifact metadata above is not publishable.

Do not treat a single synthetic benchmark as product readiness. Parser,
cascade, transform, editor, and packaging paths each need their own evidence.

## Current Committed Baseline

The current numeric authority is
[`z5-perf-gate-baseline-v0.json`](../rust/crates/omena-benchmarks/baselines/z5-perf-gate-baseline-v0.json).
It was generated on 2026-07-17 from Git commit
`99285eb305dfcc868824475d75a09a757ce45cb8` on a four-core x64 Linux host with
an Intel Xeon Platinum 8573C and approximately 16 GiB of RAM. The artifact also
records the kernel, Rust and Node versions, Cargo lock digest, comparator
version, Valgrind version, exact runner command, raw instruction counts,
comparison ratios, and thresholds.

That JSON file, not a copied Markdown table, is the number source. The scheduled
Benchmark Regression workflow regenerates it as an artifact; changing the
committed baseline remains an explicit review action rather than an automatic
rewrite from every host.

## Historical Timing Note

Earlier local Criterion and parser-product tables were captured in May 2026 on
an Apple M2 Max, but they did not carry the Git SHA required by the current
publication policy. They are intentionally omitted from the active baseline
instead of being presented as current measurements. Git history preserves the
old tables for archaeology; new reports should cite the committed baseline or a
workflow artifact with equivalent provenance.

## Parser-Product Cut-Over Guardrail

The parser-product cut-over gate compares the actual CSS Modules intermediate
producer path from raw style source to product summary. The legacy parser is an
oracle and benchmark baseline, not a product parser dependency.

This gate protects omena-css against regressions relative to its own retained
oracle lane. It is not a competitive floor, and it cannot fail merely because
an external tool is faster. Any external comparison must satisfy the same-corpus
publication policy above before it appears in positioning or release material.
