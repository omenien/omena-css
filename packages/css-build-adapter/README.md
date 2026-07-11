# @omena/css-build-adapter

Shared build adapter for the Omena CSS Vite and PostCSS integrations.

Most consumers should install `@omena/vite-plugin` or `@omena/postcss-plugin`
instead of depending on this package directly. Use this adapter when you are
building a custom integration and want the same in-process Omena CSS pipeline
used by the official build-tool plugins.

## Install

```sh
npm install -D @omena/css-build-adapter @omena/napi
```

`@omena/napi` is the preferred runtime. The adapter can fall back to
`@omena/wasm` when native bindings are unavailable and `wasmFallback` is not
disabled.

## Basic Usage

```js
const { createOmenaBuildState, runOmenaBuild } = require("@omena/css-build-adapter");

const state = createOmenaBuildState({ cwd: process.cwd() });
const result = await runOmenaBuild(
  "src/Button.module.scss",
  ".button { color: var(--brand); }",
  {
    passes: ["scss-module-evaluate"],
    minify: true,
    sourceMap: true,
  },
  state,
);

console.log(result.code);
console.log(result.map);
```

## Options

- `include` limits which files should be transformed. The default is
  `.module.css` and `.module.scss`.
- `passes` provides an explicit pass list.
- `minify`, `treeShake`, and `bundle` enable the built-in production pass
  presets.
- `sources` and `packageManifests` add workspace context for bundle-oriented
  transforms.
- `configFile` prefers the reproducible `omena.toml` config plane and keeps
  `omena.config.{ts,js,mjs,cjs,json,toml}` as compatibility inputs. Set it
  to `false` for fully explicit integration tests or build-tool adapters.
- `sourceMap` controls Source Map V3 output.

The package is part of the Omena CSS mode surface. It is not a separate bundler
product boundary.
