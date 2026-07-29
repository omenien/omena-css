---
title: Documentation handbook
description: The information architecture, ownership, authoring, review, and deployment contract for Omena documentation.
kind: how-to
status: stable
products: [documentation]
owner: developer-experience
sourceOfTruth: authored
---

# Documentation handbook

Omena keeps documentation content in `docs/` and the rendering application in
`apps/docs/`. This lets product code, generated reference, executable examples,
and release notes move in one pull request without mixing framework code into
the content authority.

## Why this structure

The layout borrows proven boundaries rather than copying one project's site:

- [Fumadocs](https://github.com/fuma-nama/fumadocs/tree/dev/apps/docs) and
  [Turborepo](https://github.com/vercel/turborepo/tree/main/apps/docs) keep a
  documentation app inside a monorepo.
- [Vite](https://github.com/vitejs/vite/tree/main/docs) and
  [Ruff](https://github.com/astral-sh/ruff/tree/main/docs) keep product
  documentation beside code so examples and generated reference can be
  validated in the same change.
- [Oxc](https://github.com/oxc-project/website) and
  [Biome](https://github.com/biomejs/website) use separate website repositories,
  which is useful when the site has an independent release and localization
  lifecycle. Omena's generated product contracts make that split premature.

The site consumes Fumadocs layout primitives instead of copying their
implementation. Small interactive controls follow
[shadcn/ui](https://ui.shadcn.com/docs/components) conventions: accessible
markup, composable variants, and source ownership limited to components Omena
actually needs.

## Choose the right page kind

Every page declares one Diataxis-style kind:

| Kind          | Reader need                         | Typical Omena page                     |
| ------------- | ----------------------------------- | -------------------------------------- |
| `tutorial`    | Learn through a guided success path | Getting started, browser playground    |
| `how-to`      | Complete a concrete task            | Editor setup, Sass adoption, migration |
| `reference`   | Look up an exact contract           | CLI, configuration, LSP capabilities   |
| `explanation` | Understand a design or tradeoff     | Positioning, performance, internals    |

Do not mix a long conceptual essay into generated reference. Link between page
kinds instead.

## Required frontmatter

Every Markdown or MDX page must declare:

```yaml
title: Human-readable page title
description: One sentence that explains the reader outcome.
kind: tutorial
status: stable
products: [cli]
owner: developer-experience
sourceOfTruth: authored
```

Allowed statuses are `stable`, `preview`, `experimental`, and `deprecated`.
`sourceOfTruth` is one of:

- `authored`: this page is maintained directly.
- `generated`: product code or a manifest produces the entire page.
- `hybrid`: authored prose contains explicitly marked generated regions.

Generated pages are changed at their producer and regenerated in the same
commit. Never patch generated output alone.

## Write durable pages

1. Search for an existing authority before adding a page.
2. Prefer one canonical explanation plus links over duplicated prose.
3. Import executable examples from tested fixtures when practical.
4. State precision limits and opt-in behavior explicitly.
5. Use product and technical names. Do not publish temporary planning labels.
6. Use repository-relative links for committed sources and normal relative
   links between documentation pages.
7. Keep the newest supported behavior as the default. Add versioned trees only
   when a maintained release line needs materially different instructions.

## Interactive examples

The browser playground is compiled from `rust/crates/omena-wasm` with
`wasm-pack --target web`. It must use public bindings rather than a
documentation-only evaluator. Keep interactive examples:

- local-only by default, with no source upload;
- bounded to browser-supported in-memory workflows;
- accessible without pointer-only controls;
- paired with prose that names filesystem or host limitations.

If an example needs package resolution, workspace discovery, or editor
lifecycle behavior, use an executable CLI or LSP fixture instead.

## Local verification

```bash
pnpm omena-check bundle docs/contracts
pnpm omena-check run docs/site
pnpm omena-check run docs/smoke
```

To exercise the browser runtime:

```bash
pnpm --filter @omena/docs build:wasm
pnpm --filter @omena/docs dev
```

The committed site build does not include generated WASM bytes. CI and the
Pages workflow build those bytes from the pinned Rust toolchain.

## Review and deployment

Pull requests declare `docs impact: paths` or `docs impact: none + reason`.
CODEOWNERS routes `docs/` and `apps/docs/` changes to the documentation owner.
Product owners review generated or hybrid pages for the contracts they own.

On `master`, the Pages workflow builds the browser WASM package, exports the
static Next.js site, uploads the immutable Pages artifact, and deploys through
the protected `github-pages` environment. The deployment does not use runtime
server actions or documentation-time API calls.
