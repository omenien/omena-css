# Benchmarks

The public benchmark story is intentionally evidence-based. Benchmark changes
must report the command, input set, machine class, and comparison baseline.

## Current Baseline Checks

- Parser product-cutover checks compare parser output against the current
  product lane.
- Runtime loop checks track request-path latency for hover, definition,
  references, and completion.
- Fuzz checks cover parser, cascade, incremental, and transform safety targets.

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

Do not treat a single synthetic benchmark as product readiness. Parser,
cascade, transform, editor, and packaging paths each need their own evidence.
