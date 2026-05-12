# Contributing

## Commit Messages

Use plain imperative commit subjects:

```text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
```

Keep commit messages understandable without private planning documents. Do not
use internal planning labels, phase names, issue-triage shorthand, or private
catalog identifiers in public history.

## Verification

Run the smallest relevant check for the files you changed, then run the broader
gate before release-oriented changes. Prefer existing `pnpm cme-check` targets
when a target exists for the changed subsystem.
