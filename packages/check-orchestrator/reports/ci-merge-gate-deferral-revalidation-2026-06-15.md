# CI Merge-Gate Deferral Revalidation

This report re-validates the user-gated merge-gate deferrals and specifies the
aggregator-only path without landing it. It is a design artifact only: no branch
protection, ruleset, `merge_group`, workflow, or orchestrator mutation is made
by this slice.

## Current-State Evidence

At the start of this pass, HEAD was `95469a1e`; the measurement artifact landed
as `53841c54`.

Read-only GitHub checks:

- `gh api repos/omenien/omena-css/branches/master/protection` returned
  `404 Branch not protected`.
- `gh api repos/omenien/omena-css/rulesets` returned `[]`.

Repository checks:

- `.github/workflows/ci.yml:16` still has unconditional
  `cancel-in-progress: true`.
- `rg --hidden "merge_group:" .github/workflows` returned no merge-queue
  trigger.
- The only `if: always()` in `ci.yml` is the closure-fast step aggregator, not a
  cross-job required-check aggregator.
- `scripts/check-rust-closure-fast-aggregation-complete.ts` verifies that
  `continue-on-error` steps are referenced in the final closure-fast failure
  fold. It does not prove that a required closure-fast gate exists in the job.
- The release publish integrity and advisory workflow-security findings from
  the earlier research are stale on this branch: `_publish-crate-train.yml` and
  `_publish-npm.yml` now contain a `release-integrity` job, and
  `workflow-security.yml` now contains an advisory `zizmor` job.
- The scheduled-escalation absence finding is also stale on this branch:
  `nightly-soak.yml`, `omena-css-drift.yml`, and `security-audit.yml` contain
  escalation steps.

## Collision Measurement

Measurement artifact:
`packages/check-orchestrator/reports/ci-merge-gate-collision-measurement-2026-06-15.json`.

Source commands:

- `gh pr list --state all --limit 100 --json number,state,createdAt,mergedAt,closedAt,headRefName,baseRefName,title`
- `gh run list --limit 100 --json databaseId,headBranch,createdAt,updatedAt,status,conclusion,event,name,headSha`

Observed window:

| Surface       | Window                                       | Count |
| ------------- | -------------------------------------------- | ----: |
| Pull requests | 2026-04-12T16:12:45Z to 2026-06-14T13:48:48Z |    34 |
| Actions runs  | 2026-06-12T11:51:22Z to 2026-06-14T22:41:34Z |   100 |

Pull-request concurrency:

| Category                  | Count | Max concurrent | Overlapping open pairs |
| ------------------------- | ----: | -------------: | ---------------------: |
| All master PRs            |    34 |              9 |                     89 |
| Dependabot master PRs     |    20 |              9 |                     89 |
| Non-dependabot master PRs |    14 |              1 |                      0 |

The high raw PR concurrency is a dependabot burst. The measured
non-dependabot PR collision rate is zero in this sample.

Actions run concurrency:

| Metric                          |                Value |
| ------------------------------- | -------------------: |
| Sampled runs                    |                  100 |
| Master push CI runs             |                   63 |
| Overlapping master push CI runs |                   19 |
| Cancelled runs                  |                   19 |
| Cancelled run class             | `CI / master / push` |

Position: `merge_group` remains deferred. The measured risk is not concurrent
PR merge collision; it is same-branch master push cancellation under
unconditional `cancel-in-progress`. The correct near-term path is:

1. Specify the cross-job aggregator now.
2. Scope `cancel-in-progress` before any required-check rollout.
3. Keep branch protection/rulesets and `merge_group` user-gated.

## Aggregator-Only Design

Target job name: `ci-required`.

Target dependency set from current `ci.yml` PR-blocking jobs:

- `verify`
- `protocol-matrix`
- `native-runner-matrix`
- `rust-workspace`
- `omena-napi-install`
- `plugin-consumers`
- `closure-fast`
- `extension-host-smoke`
- `package`

Design sketch, not landed:

```yaml
ci-required:
  if: always()
  needs:
    - verify
    - protocol-matrix
    - native-runner-matrix
    - rust-workspace
    - omena-napi-install
    - plugin-consumers
    - closure-fast
    - extension-host-smoke
    - package
  runs-on: ubuntu-latest
  timeout-minutes: 5
  steps:
    - name: Assert required CI jobs succeeded
      env:
        NEEDS_JSON: ${{ toJSON(needs) }}
      run: |
        set -euo pipefail
        printf '%s\n' "$NEEDS_JSON" |
          jq -e 'to_entries | all(.value.result == "success")'
```

The first landing should use strict `result == "success"`. If a future job is
intentionally skipped, that must be represented by an explicit skip allowlist and
a negative test proving that `failure`, `cancelled`, and unallowlisted `skipped`
do not pass.

This job is useful only after a ruleset can require it. Until then, it can be
landed as an advisory signal, but it must not be treated as branch protection.

## Required Doctor Invariant

Target diagnostic id: `workflow-required-aggregator-missing-job`.

Purpose: prevent required-set drift after `ci-required` exists.

Invariant:

- Parse `.github/workflows/ci.yml`.
- Identify every job with a blocking CI tier annotation that is not `manual`,
  `scheduled`, `release`, or `none`.
- Identify the `ci-required.needs` set.
- Emit `workflow-required-aggregator-missing-job` if any blocking job is missing
  from `ci-required.needs`.

This should extend the existing workflow manifest layer rather than duplicate it:

- `findCiTierReachabilityDiagnostics`
- `inferWorkflowJobTier`
- `# omena-ci-tier:` annotations

Golden fixture shape:

```yaml
jobs:
  verify:
    # omena-ci-tier: verify
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run core/check
  package:
    # omena-ci-tier: package
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run release/package/prepared
  ci-required:
    if: always()
    needs: [verify]
    runs-on: ubuntu-latest
    steps:
      - run: jq -e 'to_entries | all(.value.result == "success")'
```

Expected diagnostic:

```text
workflow-required-aggregator-missing-job: .github/workflows/ci.yml: ci-required.needs is missing blocking job "package".
```

Negative fold fixture:

- `{"verify":{"result":"success"},"package":{"result":"failure"}}` fails.
- `{"verify":{"result":"success"},"package":{"result":"cancelled"}}` fails.
- `{"verify":{"result":"success"},"package":{"result":"skipped"}}` fails unless
  `package` is in an explicit skip allowlist.

## Concurrency Precondition

No required aggregator should be introduced while `ci.yml` keeps unconditional
`cancel-in-progress: true`.

Target shape, not landed:

```yaml
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name != 'merge_group' && github.ref_name != 'master' && github.ref_name != 'next' }}
```

If a merge queue is later adopted, `merge_group:` must be added to the workflow
trigger in the same rollout that enables a queue-required check. Queue skipped
checks can be treated as passed by GitHub, so the aggregator fold must inspect
the dependency results itself and require success.

## Closure-Fast Flatten Decision

Do not flatten `closure-fast` into the cross-job aggregator.

Rationale:

- The current closure-fast pattern already has scar tissue around silent reds and
  late aggregation.
- The existing meta-gate protects wiring references, not required coverage.
- A cross-job `ci-required` aggregator captures the branch-protection value
  without rewriting the closure-fast internal structure.

Contract for any future rewrite of
`scripts/check-rust-closure-fast-aggregation-complete.ts`:

- Ship a non-vacuous negative self-test in the same commit.
- The negative test must remove a required outcome reference or required gate
  from a fixture and prove that the checker fails.
- Echo-only references must not satisfy the test; the outcome must participate
  in the failure fold.

## Deferred Item Revalidation

| Item                                      | Current verdict                                    | Evidence                                                                                                                        | Next action                                                                  |
| ----------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Branch protection / rulesets              | Still user-gated                                   | Branch protection GET returns 404; rulesets GET returns `[]`.                                                                   | Do not mutate without explicit user trigger.                                 |
| Cross-job aggregator                      | Ready to design, not landed                        | No `ci-required` job exists; only closure-fast has a local `if: always()` step.                                                 | Later landing goal may add advisory `ci-required` plus doctor invariant.     |
| Release publish integrity                 | Closed on this branch, pending merge               | `_publish-crate-train.yml` and `_publish-npm.yml` contain `release-integrity` before publish jobs.                              | No further action in this goal. Verify again after merge.                    |
| Workflow security / zizmor                | Advisory lane exists on this branch, pending merge | `workflow-security.yml` contains `zizmorcore/zizmor-action` with `continue-on-error: true`.                                     | Keep advisory until findings are clean or explicitly accepted.               |
| Closure-fast meta-gate coverage blindness | Still holds                                        | The checker verifies referenced outcomes for existing `continue-on-error` steps; it does not own a required coverage inventory. | Do not flatten now; require negative self-test if rewritten.                 |
| Scheduled escalation absence              | No longer holds on this branch                     | `nightly-soak.yml`, `omena-css-drift.yml`, and `security-audit.yml` contain escalation steps.                                   | No action in this goal.                                                      |
| `merge_group` trigger                     | Still deferred                                     | No branch protection/rulesets; non-dependabot PR overlap is zero in measured sample.                                            | Reconsider only with branch protection plus sustained PR collision evidence. |
| Unconditional CI cancellation             | Needs future fix before required aggregator        | 19 sampled `CI / master / push` cancellations; `ci.yml` still has unconditional cancel.                                         | Scope cancel before or with any required-check rollout.                      |

## Landing Gates For A Later Implementation

- `G1`: `omena-check doctor` emits
  `workflow-required-aggregator-missing-job` when any blocking `ci.yml` job is
  absent from `ci-required.needs`.
- `G2`: The aggregator fold requires `result == "success"` unless a job is in an
  explicit skip allowlist; synthetic `failure`, `cancelled`, and unallowlisted
  `skipped` inputs fail.
- `G3`: No ruleset or branch-protection requirement references `ci-required`
  while `ci.yml` has unconditional `cancel-in-progress: true`.

## Conclusion

The measured data does not justify landing `merge_group` now. It does justify
keeping a narrowly scoped aggregator-only plan ready, with cancel scoping as a
strict precondition and branch-protection/ruleset changes held for an explicit
user-triggered repo-admin goal.
