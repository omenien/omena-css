# Getting Started

This walkthrough goes from install to a configured workspace in a few minutes.
For a single-command taste of the product, the README's 60-second check is the
shortest path; this page continues past it.

## 1. Install The CLI

```bash
cargo install omena-cli --locked
```

Editor users can install the
[VS Code extension](vscode-extension.md) instead; both run the same engine.

## 2. Lint A File With Zero Config

Every product verb works without configuration:

```bash
omena lint src/button.module.css
```

Findings are typed and carry evidence — the report names the rule, the span,
and why the engine believes it (for example, a `missing-keyframes` finding
names the animation reference that has no matching `@keyframes` in scope).

## 3. Add A Workspace Config

Create `omena.toml` at the workspace root. The fastest start is extending a
built-in persona preset, then overriding only what you need:

```toml
extends = "omena:workspace-maintenance"

[lint]
profile = "strict"
```

Presets choose sensible verb defaults per audience — see the
[persona reference](reference/personas.md) and the
[configuration reference](reference/configuration.md) for every key. Public
TOML fences in these docs are executed against the real config parser in CI.

## 4. Ask The Workspace Questions

```bash
omena modules src/
omena explain src/button.module.css
```

`modules` reports CSS Modules structure — exports, `composes` closure, and
cross-file usage. `explain` shows why a diagnostic or transform decision was
made, including the guarantee tier the evidence earns.

## 5. Where To Go Next

- [SDK workflows](sdk.md) — the same engine from NAPI, WASM, CLI, and LSP.
- [Sass compatibility](sass-compat.md) — SIF adoption and provenance tiers.
- [Codemods](migrate-verb.md) — plan-first migrations with typed rollback.
- [Editor setup](clients/README.md) — VS Code, Zed, Neovim, or any LSP client.
