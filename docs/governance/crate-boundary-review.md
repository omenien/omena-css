# Crate boundary review

Omena product capabilities begin behind the shared query and product egress. A capability becomes a separate crate only when measured ownership pressure justifies the additional public API, dependency, build, and release boundary.

Every boundary review must provide all four measurements below. A decision without executable measurements is incomplete.

| Criterion             | Required measurement                                                                                                                                                               | Required response                                                                                                            |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| API surface stability | Count commits touching the candidate boundary from a named base revision and identify the public-surface snapshot or manifest that detects drift.                                  | Explain how accepted API churn will be reviewed and which gate fails on unreviewed drift.                                    |
| Dependency direction  | Derive direct workspace dependencies and consumers from `cargo metadata` or an equivalent source graph. Report cycles and dependencies that point back into a higher-level facade. | Explain why the boundary improves directionality or name the condition that will cause it to be folded.                      |
| Build cost            | Record a reproducible warm command sample and a successful CI job envelope. Warm timings are diagnostic, not portable performance claims.                                          | Explain how the boundary contains rebuild cost or define the threshold that triggers a new review.                           |
| Consumer count        | Derive consumers from Cargo metadata or source imports and record the complete set.                                                                                                | Explain why the number and diversity of consumers warrants the boundary or define the migration needed before consolidation. |

## Decision contract

Each review ends in exactly one state:

- `promote`: introduce a physical crate boundary in a separately reviewed change.
- `retain`: keep the current boundary because the measurements justify it.
- `revisit`: keep the current topology until every named re-review condition is measurable and satisfied.

An unfavorable measurement cannot be omitted. Every criterion includes a response, and `revisit` requires concrete conditions rather than an unspecified future decision. This review records a topology decision only; it never performs the topology change itself.

The machine-readable authority is [`rust/product-surface-boundary-reviews.json`](../../rust/product-surface-boundary-reviews.json). Run `pnpm omena-check run rust/product-surface-boundary-reviews` to recompute deterministic measurements. Pass `--measure` to its underlying script when refreshing local timing samples.
