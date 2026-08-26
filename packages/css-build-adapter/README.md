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

## Cache identity

`rebuildAndCache` serves a cached result only after the native engine seals the
current target, additional style sources, package manifests, static config,
resolver generation, target-data snapshot, engine ABI, and pass plan into one
BLAKE3 build-snapshot digest. File paths and mtimes are not sufficient cache
identity: changing dependency bytes at the same path forces a rebuild.

JSON and TOML configuration files participate by content. Dynamic JS, MJS,
CJS, and TypeScript configs can read transitive inputs that the adapter cannot
enumerate, so those shapes rebuild instead of taking a hopeful cache hit.

The `engine` option is an advanced custom-integration and test seam. An
injected engine is responsible for returning an authoritative
`buildSnapshotIdentity`, including its own ABI identity. Do not reuse one build
state while replacing it with a different injected engine that reports the same
digest. The official N-API engine binds its ABI in the native receipt; engines
without the identity endpoint rebuild instead of serving a cache hit.

Repeated `sources` paths are deduplicated before sealing. Repeated
`packageManifests` paths currently make native sealing fail closed, so the build
remains fresh but bypasses the cache until the caller removes the duplicate.
