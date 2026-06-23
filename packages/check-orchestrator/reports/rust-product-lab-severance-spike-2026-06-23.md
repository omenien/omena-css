# Rust Product/Lab Severance Spike

Date: 2026-06-23

## Scope

This spike records the crate-level severance plan for keeping the shipped
product closure free of the five current research/lab crates:

- `omena-smt`
- `omena-categorical`
- `omena-variational`
- `omena-ensemble`
- `omena-rg-flow`

The current tripwire is `pnpm omena-check run rust/product-lab-closure`. It is
report-only today and reports all five crates in both `omena-lsp-server` and
`omena-cli` normal dependency closures under `--no-default-features`.

## Current Edges

`omena-checker` currently has non-optional normal dependencies on:

- `omena-categorical`
- `omena-rg-flow`
- `omena-smt`
- `omena-variational`

`omena-query-checker-orchestrator` currently has non-optional normal
dependencies on:

- `omena-categorical`
- `omena-ensemble`

The product roots reach these crates through `omena-query` ->
`omena-query-checker-orchestrator` -> `omena-checker`, with additional transform
runner paths reaching `omena-smt` through `omena-transform-passes`.

## Severance Disposition

| Crate               | Current Product Use                                                                                                                                          | Default Verdict Risk                                                                                   | Disposition                                                                                                                                                                                                                         |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `omena-smt`         | Default `StubSmtBackendV0` discharges canonical box-shorthand obligations and can emit `cascade.smt-violation`.                                              | High: this is the default `Warning` path.                                                              | Move the propositional canonical-input evaluator and default backend result types into product-owned checker code. Keep z3/layer-inversion search in `omena-smt` behind `smt-z3`. Do not gate the default diagnostic off.           |
| `omena-categorical` | Checker and orchestrator expose categorical evidence helpers and design-system model helpers; checker can emit `categorical-cascade-evidence-inconsistency`. | Medium: emitted rule is `Hint`, but public helper exports may be consumed by product APIs.             | Put categorical helpers behind a default-off checker/orchestrator feature. If the default diagnostics corpus changes, move only the product-needed evidence summary into product-owned checker code before removing the dependency. |
| `omena-variational` | Checker uses designer-intent posterior logic to emit `designer-intent-inconsistency` for cascade source-order ties.                                          | Medium-high: emitted rule is `Hint`, but it is a default product verdict when the tie pattern appears. | Split the default product decision from the lab posterior machinery. Preserve the emitted Hint byte-for-byte via product-owned fallback or moved minimal evaluator; gate the richer variational crate default-off.                  |
| `omena-ensemble`    | Orchestrator re-exports replica-ensemble report types and checker can emit `replica-ensemble-inconsistency`.                                                 | Medium: rule is `Hint`; current product path is mostly orchestration/report wiring.                    | Gate ensemble report APIs behind a default-off orchestrator feature. Preserve default diagnostics through the severance differential oracle before removing the non-optional dependency.                                            |
| `omena-rg-flow`     | Checker exports RG-flow metadata and computes `rg-flow-relevant-operator` Hint evaluations.                                                                  | Medium: rule is `Hint`; constants are re-exported into checker reports.                                | Gate RG-flow evaluation behind a default-off checker feature. If report constants are still needed by default DTOs, move those constants into checker-owned metadata and leave numeric RG-flow analysis in the lab crate.           |

## Required Implementation Order

1. Add the default-build severance differential oracle before deleting or
   gating any behavior. The oracle must use the unchanged style-diagnostics
   corpus and compare emitted diagnostics byte-for-byte.
2. Move the default SMT propositional evaluator before feature-gating
   `omena-smt`, otherwise the default warning path would be dropped.
3. Gate the Hint-class crates behind explicit default-off features only after the
   differential oracle is in place.
4. Re-run `pnpm omena-check run rust/product-lab-closure` in report-only mode
   after each crate edge is changed.
5. Flip the closure gate to hard-fail only after the five crates are absent from
   both product roots and the severance differential oracle is green.

## Acceptance Evidence For The Next Slice

- `pnpm omena-check run rust/product-lab-closure` reports no present lab crates
  for `omena-lsp-server` or `omena-cli`.
- `cargo tree -e normal --no-default-features -p omena-lsp-server -i <labcrate>`
  and the same command for `omena-cli` return the Cargo "did not match any
  packages" absence condition for all five crates.
- The default diagnostics corpus is byte-identical before and after severance.
- The default box-shorthand SMT warning and the designer-intent Hint are still
  emitted by the default product build where the fixture exercises them.
