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
pnpm omena-check affected --base=origin/master
pnpm omena-check probe
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
public check names stay flat. Declared gates with a CI tier are also checked
against workflow reachability so a gate cannot claim `closure-fast` or `verify`
coverage while being absent from that workflow tier.

`surface` prints the current gate count, alias-chain count, and largest bundles
by unique leaf dependencies. Use it before broad gate rewrites to identify the
smallest stable surface needed by the next migration.

`CHECKS.md` is generated from the manifest. Update it with
`pnpm omena-check inventory --write` after adding, renaming, or regrouping check
scripts.

## Focused CI feedback

`affected` classifies committed and working-tree changes and recommends the
smallest registered probe profiles that cover the edited product area. Unknown
paths and workflow topology changes fail closed by requiring the complete CI
graph. The final merge-boundary run remains authoritative even when every
focused probe passes.

Run a profile locally when the host supports it:

```sh
pnpm omena-check affected --base=origin/master
pnpm omena-check probe rust-cli
```

When evidence needs a Linux, Windows, or macOS GitHub runner, dispatch the same
profile from the committed `HEAD` through the dedicated scratch ref:

```sh
pnpm ci:probe -- linux-benchmark
pnpm ci:probe -- cross-platform-cli --watch
```

The remote helper does not include uncommitted files and does not wait unless
`--watch` is supplied. It updates `codex/ci-probe`, which does not trigger the
full push workflow, and dispatches the allowlisted `CI Probe` workflow. Group
coherent local commits and run the complete CI graph once at the final boundary
instead of using full `master` pushes as an interactive debugger.
