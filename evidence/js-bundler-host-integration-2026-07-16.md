# JS Bundler Host Integration Evidence

Date: 2026-07-16

## Scope and authorities

The JS bundler boundary now transports the existing semantic CSS Modules interface instead of deriving an identity map by scanning emitted CSS. The implementation reuses these authorities:

- `OmenaQueryCssModuleInterfaceV0` and `summarize_omena_query_module_interface_projection` for module exports.
- `OmenaQueryClosedWorldOutcomeV0` for bundle admission and blockers.
- `OmenaPlugin` and the existing plugin ABI census for host registration.
- `contracts/engine-sdk-workflow/main.tsp` and `scripts/generate-engine-v2-contract-idl.ts` as the single request/response generator.

No crate was added. Protocol version `0` and the plugin ABI version remain separate authorities.

## Commit ledger

| Commit                                     | Subject                                                   | Responsibility                                                                                |
| ------------------------------------------ | --------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `63ee50aff481edede13255a9c9528ec9c131016a` | `test(adapter): capture class map parity corpus`          | Records six legacy regex maps before removal.                                                 |
| `ea9a88982aa0061d0a4a12f8546476864c46a69a` | `feat(adapter): expose semantic CSS module host protocol` | Adds generated protocol DTOs and NAPI/WASM transport of the semantic interface.               |
| `103166085e1c914e1f3a5afac27f2137174ec253` | `feat(vite): consume semantic CSS module exports`         | Removes the regex map, adds typed exports, three-way HMR, and independent parity gates.       |
| `b9f4983277a5d28cc16bc5989d48fbbc9ca4e037` | `feat(plugin): register Vite bundle host kind`            | Adds the governed bundle-host kind, residual ledger, and scheduled browser job.               |
| `882c5a8a4b8df5187a5fa06f9e4e1982fbedc3a3` | `fix(wasm): preserve bundler host records`                | Uses JSON-compatible WASM serialization for map-shaped host records.                          |
| `517c22f757ecd9b2932e25ab2c3fcf2ebb7295af` | `fix(adapter): isolate PostCSS module transforms`         | Keeps module-interface projection on bundler hosts without changing PostCSS transform output. |
| `c94ebeff4056e21a7d1017796166554e17d306ae` | `fix(sdk): synchronize bundler host boundary contracts`   | Synchronizes generated response, error, FFI, and public-surface records after host exposure.  |
| `ee67404076774ef8590a61f8ae7efcc4dfcdb988` | `fix(adapter): enforce typed bundle admission`            | Transports existing bundle outcome/evidence and rejects open or incomplete bundle admission.  |
| `1e11d7012595279c524865f561c4582c767757a5` | `test(adapter): harden bundler host closure proofs`       | Adds real TypeScript consumers and load-bearing composes, parity, and entry-point faults.     |
| `9c425c03d64259b36fec284bf0afd6ce1845b178` | `fix(adapter): generate bundler host declarations`        | Removes the handwritten package DTO copy and emits the adapter declaration from TypeSpec.     |
| `c191d4aabd659a2258478041e6d002c3eb230f76` | `fix(checks): harden bundler host boundary validation`    | Scans the complete package source boundary and compiles consumers without a direct TS API.    |

The regex class-map implementation was removed in `103166085e1c914e1f3a5afac27f2137174ec253` after the independent parity gate was green.

## Legacy mismatch witness

`scripts/fixtures/css-module-host-parity/cases.json` preserves six pre-removal outputs. Every legacy result maps emitted names to themselves and differs from the semantic interface. Representative cases:

| Case                  | Legacy regex map                                               | Semantic map                                                  |
| --------------------- | -------------------------------------------------------------- | ------------------------------------------------------------- |
| CSS local             | `{ "_root_0": "_root_0" }`                                     | `{ "root": "_root_0" }`                                       |
| CSS local composes    | `{ "_button_0": "_button_0", "_base_1": "_base_1" }`           | `{ "base": "_base_1", "button": "_button_0 _base_1" }`        |
| CSS imported composes | `{ "_button_0": "_button_0" }`                                 | `{ "button": "_button_0 _base_0" }`                           |
| SCSS nested           | `{ "_card_0": "_card_0", "_card__title_1": "_card__title_1" }` | `{ "card": "_card_0", "card__title": "_card__title_1" }`      |
| SCSS local composes   | `{ "_panel_0": "_panel_0", "_surface_1": "_surface_1" }`       | `{ "panel": "_panel_0 _surface_1", "surface": "_surface_1" }` |
| Less local composes   | `{ "_chip_0": "_chip_0", "_tone_1": "_tone_1" }`               | `{ "chip": "_chip_0 _tone_1", "tone": "_tone_1" }`            |

The parity command uses two distinct product paths: `omena modules emit` for the build side and the adapter through `engine-shadow-runner bundler-host-resolve-module` for the development side. It does not compare two references to the same in-memory object.

```text
$ node --import tsx ./scripts/check-js-bundler-host-parity.ts
fixtureCount: 6
css-local-class: parity=true
css-local-composes: parity=true
css-imported-composes: parity=true
scss-nested-class: parity=true
scss-local-composes: parity=true
less-local-composes: parity=true
```

Generated declarations are compared byte-for-byte by the parity gate. Every fixture also creates a real TypeScript consumer that imports both the default map and all safe named exports, then runs the TypeScript compiler with strict checking and bundler module resolution. This guards the consumer-facing declaration shape rather than only comparing declaration strings.

## Protocol and export behavior

The generated host response carries `classMap`, `namedExports`, declaration text, composes edges, diagnostics, and a snapshot identifier. If the semantic view cannot be produced, the adapter returns a typed failure; there is no CSS-text fallback.

Vite exports the semantic default map and safe named exports. HMR classifies changes into three decisions:

- `styleOnly`: keys and values are unchanged; only CSS changed.
- `valueChanged`: keys are unchanged but scoped values changed; the new map is pushed to consumers.
- `shapeChanged`: keys changed; the module and its importers are invalidated without a blanket page reload.

The browser smoke imports both the default map and named `root`, asserts `_root_0`, performs rapid edits, and rejects browser exceptions, console errors, and page reloads.

## Falsification transcripts

Each mutation below was applied to the production or generated source, observed RED, reverted, and followed by a GREEN rerun.

### Stale-map routing

Mutation: route a value-only export delta to `styleOnly`.

```text
AssertionError: expected 'styleOnly' to be 'valueChanged'
```

After restoring the three-way decision, the Vite and adapter suites passed `12/12`.

### Regex resurrection

Mutation: add a `.matchAll(...)` CSS scanner to the adapter.

```text
AssertionError: bundler host adapters must not derive class maps by scanning emitted CSS
```

After removal:

```json
{
  "schemaVersion": "0",
  "product": "js-bundler-host.no-regex-classmap",
  "scannedFiles": 5,
  "semanticTransportAnchors": 3
}
```

The standing gate recursively scans all JavaScript and TypeScript sources under both host packages rather than pinning only their entry files. Its permanent `--inject-regex-classmap` fault adds a scanner in a synthetic helper and is rejected before semantic transport assertions run.

### Ungoverned host registration

Mutation: classify `vite-bundle-host` as `Transform` instead of `BundleHost`.

```text
AssertionError: expected exactly one bundle-host registration, received 0
```

After restoration:

```json
{
  "pluginKinds": ["transform", "bundleHost"],
  "bundleHostRegistrationCount": 1,
  "versionAuthoritiesDistinct": true,
  "residualEntryCount": 6
}
```

### Generated contract drift

Mutation: change the bundler-host protocol version in TypeSpec without regenerating outputs.

```text
generated contract check reported a diff in the generated TypeScript protocol
```

After regeneration and restoration, `generate-engine-v2-contract-idl.ts --check` completed successfully and reported all 18 generated files synchronized, including `packages/css-build-adapter/bundler-host-contract.generated.d.ts`.

### Composes-edge omission

Mutation: remove the imported `composes` edge from the `css-imported-composes` fixture while retaining the committed semantic expectation.

```text
AssertionError: Expected values to be strictly deep-equal:
actual:   { button: '_button_0' }
expected: { button: '_button_0 _base_0' }
```

The normal six-fixture run restores the composed class list and typechecks the generated consumer.

### Development-map rename

Mutation: rename one class only in the development host response.

```text
AssertionError: Expected values to be strictly deep-equal:
actual:   { root: '_root_0 renamed' }
expected: { root: '_root_0' }
```

This proves the development side is compared against the independent CLI emit path rather than another reference to the same host response.

### Unregistered native host entry point

Mutation: inject a second package path that opens the native bundler-host boundary directly.

```text
AssertionError: native bundler-host access must stay behind the registered shared adapter entry point
actual: packages/css-build-adapter/index.cjs, packages/unregistered-bundle-host/index.cjs
expected: packages/css-build-adapter/index.cjs
```

The normal scan derives exactly one direct boundary owner from package source files; the count is not a hard-coded success value.

## Integration defects found during validation

### WASM record serialization

Rust `BTreeMap` values initially crossed `serde_wasm_bindgen` as JavaScript `Map` objects. JSON/object consumers therefore observed empty `classMap` and `namedExports` records even though declaration text was correct. The bundler-host WASM functions now use `Serializer::json_compatible()`.

Observed after rebuilding the local WASM artifact:

```json
{ "classMap": { "root": "_root_0" }, "namedExports": { "root": "_root_0" }, "ready": true }
```

### PostCSS transform isolation

The shared adapter initially enabled semantic class hashing for the PostCSS transform surface, changing its established selector-preserving output. PostCSS now opts out with `moduleInterface: false`; Vite and direct bundler-host calls retain semantic module behavior. Both the full Vite HMR smoke and PostCSS consumer smoke pass together.

### Typed bundle admission

The adapter's `bundle: true` path previously accepted a regular multi-source build when the engine lacked a bundle callable, and the NAPI/WASM bundle response exposed only the CSS artifact. That dropped the existing closed-world outcome, decision parity, and evidence manifest before the JavaScript host could enforce them.

The existing bundle callable now returns the artifact fields together with the existing typed outcome, parity, and evidence values. The adapter passes the actual target as the bundle entry, requires the bundle callable, and rejects open or incomplete admission before returning CSS. A unit fixture supplies an `open` outcome with a typed missing-dependency blocker and asserts that the partial CSS never becomes a build result. The NAPI callable test also parses the real serialized response and verifies the flattened artifact plus `closedWorldOutcome`, parity, and evidence fields.

### Generated package declaration authority

The first package declaration exposed the correct response shape but copied three protocol interfaces by hand. That violated the single-generator contract even though runtime serialization and typechecking were green. The SDK workflow generator now emits the adapter-facing declaration directly from `contracts/engine-sdk-workflow/main.tsp`; `index.d.ts` only imports and re-exports those generated types. The plugin-kind gate rejects a handwritten `OmenaBundlerHostResolveModuleResponseV0`, composes edge, or diagnostic interface in the package entry declaration.

### TypeScript consumer execution

The real consumer gate initially imported the TypeScript compiler API directly. Push CI run [29434069711](https://github.com/omenien/omena-css/actions/runs/29434069711) correctly rejected that bypass in the `package` job while every other completed CI job was green. The gate now invokes the repository `tsc` executable with strict checking and bundler module resolution for every generated consumer. This keeps the consumer proof real without expanding the engine's classic TypeScript API façade; `ts7/ts-api-surface-lock` and all six parity fixtures pass locally.

## Residual ledger

`rust/omena-bundler-host-residual-ledger.json` records all deferred or partial host work:

| Residual                     | Status   | Owner                           |
| ---------------------------- | -------- | ------------------------------- |
| webpack host                 | deferred | css-build-adapter host adapters |
| Rspack host                  | deferred | css-build-adapter host adapters |
| chunk graph                  | unowned  | unowned                         |
| cross-chunk CSS order        | deferred | bundle emission-order contract  |
| SSR/module federation        | deferred | bundler host runtime adapters   |
| source-map passthrough depth | partial  | css-build-adapter               |

The plugin-kind gate requires this exact six-row ledger and requires the chunk graph row to remain explicitly unowned.

## Local verification

Green checks include:

- `cargo test -p omena-query-transform-runner -p omena-query`
- `cargo test -p omena-wasm`
- strict clippy for `omena-query-transform-runner` and `omena-wasm`
- repository pre-push Rust clippy, Rust fmt, and TypeScript formatting
- generated TypeSpec contract check
- TypeScript API surface lock
- adapter/Vite unit tests: 13 passed
- six-fixture independent parity
- no-regex and plugin-kind gates
- plugin ABI and consumption-law gates
- orchestrator inventory/check/doctor and `rust/closure-fast`
- Vite smoke, full local Vite+Chrome HMR, PostCSS consumer smoke, package staging, bundler product gate, and oxlint smoke
- `core/check`

A package-only local NAPI artifact built by Xcode 27 was rejected as malformed Mach-O (`mis-aligned LINKEDIT string pool`). The same checkout passed through the current WASM host boundary locally. The scheduled Linux runner builds the NAPI artifact independently and is the authoritative cross-platform proof.

Full strict clippy for `omena-query` still reports 22 pre-existing unused/dead-code warnings in untouched style modules. Goal-owned crates and the repository pre-push clippy gate are green; those unrelated warnings were not suppressed or expanded here.

## Push-CI contract synchronization

The first code push exposed four stale closed-world records rather than a runtime failure:

- FFI boundary typing census omitted the two NAPI and two WASM host exports.
- SDK error mapping census omitted three new host serialization/error sites.
- The generated workflow response gate still expected five public responses instead of six.
- The `omena-query` public API snapshot did not yet contain the new protocol and module-interface fields.

The scan-derived censuses and public API snapshot were regenerated and reviewed. Invalid request JSON is classified as `input` on both NAPI and WASM surfaces; the shared JSON-compatible serializer is `internal`. The typed bundle transport adds one transform-domain error site per FFI surface without adding a callable. Local reruns then passed with 64 FFI callables, 58 SDK error sites, six public responses, and the complete `omena-query` boundary bundle.

## CI and scheduled browser proof

- Initial code-push CI: [run 29427882117](https://github.com/omenien/omena-css/actions/runs/29427882117) exposed the four stale contracts above; all product and cross-platform build jobs, including Linux NAPI installation, passed.
- Workflow Security: [run 29427881540](https://github.com/omenien/omena-css/actions/runs/29427881540) - success.
- Release Plan: [run 29427881598](https://github.com/omenien/omena-css/actions/runs/29427881598) - success.
- Nightly Soak dispatch: [run 29427886646](https://github.com/omenien/omena-css/actions/runs/29427886646) - `vite-bundler-host-hmr` succeeded after building the Linux NAPI artifact and running Chrome 150. The overall scheduled run remained red only in the standing `selected-query-default` and `checker-release-gate` jobs; the new host job was green.
