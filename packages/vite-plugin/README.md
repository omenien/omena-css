# @omena/vite-plugin

Vite consumer surface for the in-process Omena CSS native/wasm build API.

## Install

```sh
npm install -D @omena/vite-plugin@0.2.1 @omena/napi@0.2.1 vite@^8
```

`@omena/napi` is the preferred runtime. The plugin can fall back to
`@omena/wasm` when native bindings are unavailable and `wasmFallback` is not
disabled.

## Vite Config

```js
import { omenaCss } from "@omena/vite-plugin";

export default {
  plugins: [
    omenaCss({
      passes: ["comment-strip", "whitespace-strip"],
      minify: true,
      treeShake: true,
      bundle: true,
      sources: ["src/styles/tokens.module.css"],
    }),
  ],
};
```

## Scope

The published `0.2.1` scope is intentionally conservative:

- `.module.css` and `.module.scss` files are transformed by default
- the hot path calls `@omena/napi` directly and falls back to `@omena/wasm`
  when native bindings are unavailable
- CLI and `cargo run` fallback are intentionally not used in Vite transforms
- Vite dev serves an Omena-owned CSS Modules runtime with `import.meta.hot`
  acceptance so style edits update without a full page reload
- additional style sources, package manifests, and static config files are
  registered with Vite's watcher; their edits rebuild every cached target whose
  native build-snapshot receipt names that dependency
- `treeShake`, `bundle`, and `minify` compose built-in pass presets; provide
  `sources`/`packageManifests` when bundle context needs additional workspace
  files
- `omena.config.{ts,js,mjs,cjs,json,toml}` can provide defaults; explicit
  plugin options override config values

Use `include` to opt into a wider path set after the downstream build pipeline
is ready for omena-owned preprocessing.

Repository source also recognizes `.module.less`, but that behavior is not part
of the published `0.2.1` contract. Check the installed package version before
relying on repository-only defaults.

## Transform Input Provenance

Repository source analyzes the current Vite transform input by default. This
lets Omena consume a style source produced by an earlier plugin instead of
silently falling back to the file on disk. Set `requireDiskSource: true` to opt
into strict mode, where the plugin skips transform input that differs from its
disk file or has no corresponding disk file.

Each cached build summary reports the input's provenance without introducing a
second source-identity hash:

| Classification     | Meaning                                                       |
| ------------------ | ------------------------------------------------------------- |
| `disk-backed`      | The transform input is byte-identical to the disk file.       |
| `virtual-with-map` | The input differs and has a usable upstream combined map.     |
| `virtual-only`     | The input differs, or has no disk file, without a usable map. |

The summary includes `classification`, the BLAKE3 `inputDigest`, a nullable
`diskDigest`, `upstreamMapPresent`, and `upstreamMapSources`. The input digest is
the target-source digest already owned by the build-snapshot identity. A source
map is accepted as upstream provenance only when it maps back to the matching
disk contents; Vite's synthesized identity fallback is not presented as proof
of an upstream transform.

Changing the default from strict disk matching to transform-input analysis is a
user-visible behavior change intended for the next pre-1.0 minor release. This
repository change does not publish a package; publication remains part of the
normal release train. The checked-in
[`virtual-source-admission-census.json`](./virtual-source-admission-census.json)
separates existing disk-backed inputs from newly admitted mapped and virtual-only
inputs, and the smoke gate rejects unreviewed census drift.
