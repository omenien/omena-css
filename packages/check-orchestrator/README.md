# Check Orchestrator

Internal gate inventory and runner for Omena CSS Modules.

This package maintains the typed gate inventory for Omena CSS Modules. It still
mirrors root `package.json` scripts, and it can also load declared gates whose
commands, dependencies, CI tier, and compatibility aliases are modeled directly
in the orchestrator. CI and release verification can route through the
manifest-backed CLI while legacy script names stay valid:

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

Root scripts remain the compatibility surface for package-derived gates, but
migrated gates should use declared manifest metadata as their source of truth.
Aggregate root scripts and workflows should depend on canonical `omena-check`
gate IDs instead of chaining legacy `check:*` script names directly. The
orchestrator layer provides stable gate IDs, grouping, bundle introspection,
argument forwarding, execution plans, and doctor checks so workflows do not need
to duplicate every script name.
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
