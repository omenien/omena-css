# Check Orchestrator

Internal gate inventory and runner for CSS Module Explainer.

This package mirrors the existing root `package.json` scripts into typed gate
metadata without removing the old script names. CI and release verification can
route through the manifest-backed CLI while the legacy script names stay valid:

```sh
pnpm omena-check list
pnpm omena-check run core/check
pnpm omena-check bundle rust/release/bundle
pnpm omena-check bundle tsgo/release/bundle
pnpm omena-check bundle release/release/verify
pnpm omena-check plan release/release/verify
pnpm omena-check doctor
pnpm omena-check surface
pnpm omena-check inventory --check
```

The root scripts remain the executable source of truth. Aggregate root scripts
should depend on canonical `omena-check` gate IDs instead of chaining legacy
`check:*` script names directly. The orchestrator layer provides stable gate IDs,
grouping, bundle introspection, argument forwarding, execution plans, and doctor
checks so workflows do not need to duplicate every script name.
`doctor` also rejects GitHub workflow calls that bypass `omena-check` for
manifest-covered package scripts, non-canonical or unknown `omena-check` targets,
and `bundle` calls pointed at non-bundle gates. It warns on alias chains so
public check names stay flat.

`surface` prints the current gate count, alias-chain count, and largest bundles
by unique leaf dependencies. Use it before broad gate rewrites to identify the
smallest stable surface needed by the next migration.

`CHECKS.md` is generated from the manifest. Update it with
`pnpm omena-check inventory --write` after adding, renaming, or regrouping check
scripts.
