---
title: Lint integrations
description: Understand which Omena lint adapters are published and which remain repository-only.
kind: how-to
status: preview
products: [lint, eslint, stylelint, oxlint]
owner: lint
sourceOfTruth: authored
---

# Lint integrations

The CLI lint surface is the stable entry point:

```sh
omena lint .
```

The repository also contains ESLint, Stylelint, and Oxlint adapters that map
host rules to Omena diagnostic codes. These packages are not currently
published on npm. Treat their READMEs and tests as source-development
documentation, not installation instructions.

| Adapter | Registry status | Repository entry |
| --- | --- | --- |
| ESLint | unpublished | `packages/eslint-plugin` |
| Stylelint | unpublished | `packages/stylelint-plugin` |
| Oxlint | unpublished | `packages/oxlint-plugin` |

Do not install a similarly named third-party package and assume it is this
repository's adapter. A future publication must add a versioned example and
compatibility entry before this page becomes stable.
