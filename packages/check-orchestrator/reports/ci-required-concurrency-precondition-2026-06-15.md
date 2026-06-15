# CI Required Check Concurrency Precondition

This artifact specifies the concurrency precondition for a future required
aggregator. It does not land workflow changes, branch protection, rulesets, or a
merge queue trigger.

## Current Risk

Current `ci.yml` uses unconditional:

```yaml
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

The measured run sample found 19 cancelled `CI / master / push` runs among 63
sampled master push CI runs. That is a same-branch cancellation issue, not a
pull-request merge collision issue.

## Required Precondition

Before `ci-required` can become a required check, CI cancellation must be scoped
away from protected and queue-like refs.

Target shape, not landed:

```yaml
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name != 'merge_group' && github.ref_name != 'master' && github.ref_name != 'next' }}
```

If a merge queue is later adopted:

- Add `merge_group:` to the workflow trigger in the same rollout that enables a
  queue-required check.
- Keep `merge_group` runs out of cancellation.
- Do not rely on GitHub's skipped-check interpretation. The aggregator must
  inspect `toJSON(needs)` and require success, with any skip behavior controlled
  by an explicit allowlist.

## Gate For Future Landing

No ruleset or branch-protection requirement may reference `ci-required` while
`ci.yml` still has unconditional `cancel-in-progress: true`.

The future landing gate is:

1. `rg --hidden "cancel-in-progress: true" .github/workflows/ci.yml` must not
   find the unconditional shape.
2. The workflow must use a ref-scoped expression before `ci-required` is made
   required.
3. A synthetic aggregator test must prove `failure`, `cancelled`, and
   unallowlisted `skipped` dependency results fail.
