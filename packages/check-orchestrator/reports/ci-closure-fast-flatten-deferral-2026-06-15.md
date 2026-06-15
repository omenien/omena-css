# Closure-Fast Flatten Deferral

This artifact records the decision to defer flattening `closure-fast` into a
cross-job aggregator.

## Decision

Do not flatten `closure-fast`.

The future `ci-required` aggregator should depend on the existing `closure-fast`
job as one required unit instead of rewriting the internals of that job.

## Rationale

- `closure-fast` intentionally runs many internal gates as
  `continue-on-error: true` steps and folds outcomes at the end.
- The current meta-gate catches missing outcome wiring for existing
  `continue-on-error` steps.
- The current meta-gate does not prove a required coverage inventory. A gate can
  be absent from `ci.yml` and still not violate the wiring check.
- Rewriting this job is higher risk than adding a cross-job aggregator over the
  existing job set.

## Future Rewrite Contract

If `scripts/check-rust-closure-fast-aggregation-complete.ts` is rewritten, the
same commit must include a non-vacuous negative self-test.

Required negative-test behavior:

- Removing a required outcome reference from a fixture must fail.
- Removing a required gate from a fixture inventory must fail if inventory
  enforcement is added.
- Echo-only `steps.<id>.outcome` references must not count as gating.
- The test must prove the checker itself would red when the wiring or inventory
  is removed.

This keeps the closure-fast meta-gate from becoming self-validating while still
avoiding a risky job flatten.
