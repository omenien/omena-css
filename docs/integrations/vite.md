---
title: Vite integration
description: Run Omena transforms through Vite while preserving explicit workspace and HMR boundaries.
kind: how-to
status: stable
products: [vite, bundler]
owner: bundler
sourceOfTruth: authored
---

# Vite integration

Install the published plugin:

```sh
npm install --save-dev @omena/vite-plugin@0.5.0
```

```js
import { defineConfig } from "vite";
import { omenaCss } from "@omena/vite-plugin";

export default defineConfig({
  plugins: [
    omenaCss({
      minify: true,
      passes: ["comment-strip", "whitespace-strip"],
    }),
  ],
});
```

Version `0.5.0` transforms `.module.css` and `.module.scss` by default. The
repository's next adapter also includes `.module.less`; opt into a wider
published scope with `include` rather than relying on unreleased behavior.

The plugin calls NAPI first and falls back to WASM. It does not spawn the CLI.
Its development runtime accepts Vite HMR updates, while workspace-aware bundle
passes require caller-provided `sources` and `packageManifests`.

Use an explicit `include` when another preprocessor owns the same file. Avoid
running two transforms over one module unless their ordering and source maps are
tested together.
