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

The summary includes `classification`, a nullable BLAKE3 `inputDigest`, a
nullable `diskDigest`, a nullable `reason`, `upstreamMapPresent`, and
`upstreamMapSources`. The input digest is the target-source digest already owned
by the build-snapshot identity. A disk-backed input still builds when the engine
or a dynamic configuration cannot seal that identity; its digests are `null`
and `reason` names the cache-bypass cause. On an engine without build-snapshot
identity support, disk-backed and virtual inputs still build with
`inputDigest: null`, and `reason` is `engineMissingBuildSnapshotIdentity`.

A source map is accepted as upstream provenance only when one unambiguous
`sources[i]` normalizes to the transformed file, `sourcesContent[i]` is the
matching disk source, and a non-identity mapped segment names that same source.
Foreign-source and identity-only maps remain `virtual-only`; Vite's synthesized
identity fallback is not presented as proof of an upstream transform.

Changing the default from strict disk matching to transform-input analysis is a
user-visible behavior change intended for the next pre-1.0 minor release. The
transform-input default applies to the build lane; `serve` + `devRuntime`
analyzes the disk file because its Omena-owned virtual module resolves before
Vite's transform chain. The repository fixture pins that limit so build and
serve cannot be presented as equivalent. Set `devRuntime: false` only when the
downstream Vite CSS pipeline, rather than Omena's dev runtime, should own serve.

This repository change does not publish a package; publication remains part of
the normal release train. The input-unit census below loads the examples config
through Vite's resolved plugin list, executes the pre-Omena transform chain for
every tracked examples and real-project-corpus style input, and compares the
observed input with disk bytes. All 29 included `.module.scss` inputs are
rewritten with an identity-bearing map and newly admitted; the three remaining
CSS/Less inputs are outside the configured include. A separate unit fixture
pins the `virtual-only` behavior, but it is not counted as a tracked corpus
input.

<!-- omena-vite-virtual-source-admission-census:start -->

```json
{
  "schemaVersion": "1",
  "product": "omena-vite.virtual-source-admission-census",
  "package": "@omena/vite-plugin",
  "countUnit": "inputs",
  "semverIntent": "next-pre-1.0-minor",
  "laneScope": {
    "build": "resolved Vite pre-transform chain",
    "serveDevRuntime": "disk file"
  },
  "rows": [
    {
      "input": "examples/plugin-consumers/src/App.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/01-basic/Button.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/02-multi-binding/Button.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/02-multi-binding/Card.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/03-multiline/MultilineForm.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/04-dynamic/DynamicKeys.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/05-global-local/GlobalLocal.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/06-alias/Alias.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/07-function-scoped/FunctionScoped.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/08-css-only/CssOnly.module.css",
      "population": "examples",
      "included": false,
      "provenanceClass": "not-included",
      "admission": "not-included",
      "upstreamTransforms": []
    },
    {
      "input": "examples/src/scenarios/09-large/Large.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/10-clsx/Clsx.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/11-ts-path/TsPath.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/12-nested-style-facts/NestedStyleFacts.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/13-shadowing/Shadowing.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/14-non-finite-dynamic/NonFiniteDynamic.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/15-composes/Base.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/15-composes/Composes.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/16-diagnostics-recovery/BrokenComposes.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/16-diagnostics-recovery/DiagnosticsRecovery.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/16-diagnostics-recovery/DiagnosticsRecoveryBase.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/17-bracket-access/BracketAccess.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/18-less-module/LessModule.module.less",
      "population": "examples",
      "included": false,
      "provenanceClass": "not-included",
      "admission": "not-included",
      "upstreamTransforms": []
    },
    {
      "input": "examples/src/scenarios/19-keyframes/Keyframes.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/20-value/Value.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "examples/src/scenarios/20-value/ValueTokens.module.scss",
      "population": "examples",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "test/_fixtures/real-project-corpus/AnalyticsGrid.module.less",
      "population": "real-project-corpus",
      "included": false,
      "provenanceClass": "not-included",
      "admission": "not-included",
      "upstreamTransforms": []
    },
    {
      "input": "test/_fixtures/real-project-corpus/ButtonVariants.module.scss",
      "population": "real-project-corpus",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "test/_fixtures/real-project-corpus/MarketingCard.module.scss",
      "population": "real-project-corpus",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "test/_fixtures/real-project-corpus/MarketingCardBase.module.scss",
      "population": "real-project-corpus",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "test/_fixtures/real-project-corpus/StatusChip.module.scss",
      "population": "real-project-corpus",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    },
    {
      "input": "test/_fixtures/real-project-corpus/StatusChipTokens.module.scss",
      "population": "real-project-corpus",
      "included": true,
      "provenanceClass": "virtual-with-map",
      "admission": "newly-admitted",
      "upstreamTransforms": ["examples-upstream-virtual-source"]
    }
  ],
  "totals": {
    "trackedInputs": 32,
    "included": 29,
    "notIncluded": 3,
    "existing": 0,
    "newlyAdmitted": 29,
    "byProvenance": {
      "diskBacked": 0,
      "virtualWithMap": 29,
      "virtualOnly": 0
    }
  }
}
```

<!-- omena-vite-virtual-source-admission-census:end -->
