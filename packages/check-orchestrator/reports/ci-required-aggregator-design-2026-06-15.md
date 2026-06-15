# CI Required Aggregator Design

This artifact specifies the cross-job aggregator shape only. It does not land a
workflow change and does not mutate branch protection or rulesets.

## Target Job

Target job name: `ci-required`.

Blocking job set from current `ci.yml`:

- `verify`
- `protocol-matrix`
- `native-runner-matrix`
- `rust-workspace`
- `omena-napi-install`
- `plugin-consumers`
- `closure-fast`
- `extension-host-smoke`
- `package`

Design sketch:

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

The first landing should be strict-success only. Future intentional skips require
an explicit allowlist and a negative test that proves unallowlisted `skipped`,
`failure`, and `cancelled` states fail.

## Doctor Invariant

Target diagnostic id: `workflow-required-aggregator-missing-job`.

Invariant:

1. Parse `.github/workflows/ci.yml`.
2. Identify every blocking job with `# omena-ci-tier:` that is not `manual`,
   `scheduled`, `release`, or `none`.
3. Identify `ci-required.needs`.
4. Emit `workflow-required-aggregator-missing-job` for each blocking job absent
   from `ci-required.needs`.

This should extend the existing workflow manifest layer:

- `findCiTierReachabilityDiagnostics`
- `inferWorkflowJobTier`
- `# omena-ci-tier:` annotations

Golden fixture:

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

Negative fold cases:

- `{"verify":{"result":"success"},"package":{"result":"failure"}}` fails.
- `{"verify":{"result":"success"},"package":{"result":"cancelled"}}` fails.
- `{"verify":{"result":"success"},"package":{"result":"skipped"}}` fails unless
  `package` is in an explicit skip allowlist.
