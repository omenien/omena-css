# Using `omena migrate`

`omena migrate` is a plan-first codemod surface. Planning never changes source
files; applying revalidates the plan and writes through the shared FixSafety
gate.

## Supported Codemods

- `css-modules-rename`: rename a selector across its style and source uses.
- `sass-import-to-use`: replace eligible Sass `@import` rules with `@use`.
- `token-rename`: rename a CSS custom property across indexed occurrences.

## Plan

Create and inspect a deterministic JSON plan before writing:

```sh
omena migrate css-modules-rename button button-primary \
  --root . --plan migrate-selector.json --json

omena migrate sass-import-to-use \
  --root styles --plan migrate-sass.json --json

omena migrate token-rename brand-color brand-accent \
  --root . --plan migrate-token.json --json
```

A plan records `MigrationPlanV0` edits, source hashes, byte spans, evidence,
FixSafety partitions, blockers, and inverse edits. Review `reviewEdits` and
`blockers` before applying it.

## Apply

Apply a reviewed plan with the same codemod family:

```sh
omena migrate css-modules-rename --apply migrate-selector.json --json
omena migrate sass-import-to-use --apply migrate-sass.json --json
omena migrate token-rename --apply migrate-token.json --json
```

Safe edits need no additional flag. Conservative edits require explicit review:

```sh
omena migrate sass-import-to-use \
  --apply migrate-sass.json --approve-review --json
```

Applying fails closed when the plan schema or codemod differs, a blocker remains,
the source hash or expected text changed, edits overlap, evidence is incomplete,
or an edit remains manual-review-only. `--approve-review` permits conservative
edits; it does not override manual-review-only decisions.

## Rollback Evidence

The plan contains inverse edits but no pre-issued receipt. A successful apply
returns a typed receipt covering the input signature and inverse patch. Keep the
plan and apply report together when a migration must be audited or reversed.

External Sass package compatibility and lockfile adoption are documented in
[External Sass And SIF Compatibility](sass-compat.md).
