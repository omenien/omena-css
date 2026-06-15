# Benchmarks

The public benchmark story is intentionally evidence-based. Benchmark changes
must report the command, input set, machine class, and comparison baseline.

## Current Baseline Checks

- Parser product-cutover checks compare parser output against the current
  product lane.
- Runtime loop checks track request-path latency for hover, definition,
  references, and completion.
- Fuzz checks cover parser, cascade, incremental, and transform safety targets.

## Surface Disposition

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

## Reporting Template

```text
Command:
Inputs:
Machine:
Baseline:
Result:
Regression threshold:
Notes:
```

## Artifact Schema

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
