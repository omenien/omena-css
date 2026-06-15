# CI Deferral Revalidation Table

This artifact re-validates the user-gated and deferred CI items at the current
branch state. It is read-only with respect to GitHub repository settings.

| Item                                      | Current verdict                                    | Evidence                                                                                                                        | Next action                                                                  |
| ----------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Branch protection / rulesets              | Still user-gated                                   | Branch protection GET returns `404 Branch not protected`; rulesets GET returns `[]`.                                            | Do not mutate without an explicit user trigger.                              |
| Cross-job aggregator                      | Ready to design, not landed                        | No `ci-required` job exists; the only `if: always()` in `ci.yml` is the closure-fast local aggregation step.                    | Later landing goal may add advisory `ci-required` plus doctor invariant.     |
| Release publish integrity                 | Closed on this branch, pending merge               | `_publish-crate-train.yml` and `_publish-npm.yml` contain `release-integrity` before publish jobs.                              | No further action in this goal. Re-verify after merge.                       |
| Workflow security / zizmor                | Advisory lane exists on this branch, pending merge | `workflow-security.yml` contains `zizmorcore/zizmor-action` with `continue-on-error: true`.                                     | Keep advisory until findings are clean or explicitly accepted.               |
| Closure-fast meta-gate coverage blindness | Still holds                                        | The checker verifies referenced outcomes for existing `continue-on-error` steps; it does not own a required coverage inventory. | Do not flatten now; require negative self-test if rewritten.                 |
| Scheduled escalation absence              | No longer holds on this branch                     | `nightly-soak.yml`, `omena-css-drift.yml`, and `security-audit.yml` contain escalation steps.                                   | No action in this goal.                                                      |
| `merge_group` trigger                     | Still deferred                                     | No branch protection/rulesets; measured non-dependabot PR overlap is zero.                                                      | Reconsider only with branch protection plus sustained PR collision evidence. |
| Unconditional CI cancellation             | Needs future fix before required aggregator        | Sampled Actions data shows 19 `CI / master / push` cancellations; `ci.yml` still has unconditional cancellation.                | Scope cancellation before or with any required-check rollout.                |

Summary: the stale research items are release publish integrity, advisory
workflow security, and scheduled escalation. The still-active deferrals are
branch protection/rulesets, cross-job aggregator landing, closure-fast inventory
coverage, `merge_group`, and cancel scoping.
