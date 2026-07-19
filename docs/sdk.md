# SDK Workflows

omena-css exposes the same snapshot-bound workflows through NAPI, WASM, the
`omena` process, and the Rust LSP. The IDL in
`contracts/engine-sdk-workflow/main.tsp` owns request, response, partition, and
typed error envelopes.

## Install

```bash
npm install @omena/napi
npm install @omena/wasm
cargo install omena-cli --locked
```

## Workflow Matrix

This table is generated from the TypeSpec request models and checked against the
committed four-surface parity matrix.

<!-- BEGIN GENERATED: OMENA SDK WORKFLOWS -->
<!-- Generated from product code. Do not edit by hand. -->
| Workflow | Covered surfaces |
| --- | --- |
| `snapshot` | `napi`, `wasm`, `cli`, `lsp` |
| `query` | `napi`, `wasm`, `cli`, `lsp` |
| `diagnostics` | `napi`, `wasm`, `cli`, `lsp` |
| `build` | `napi`, `wasm`, `cli`, `lsp` |
| `explain` | `napi`, `wasm`, `cli`, `lsp` |
<!-- END GENERATED: OMENA SDK WORKFLOWS -->

Every operation after `snapshot` carries its `snapshotId`. A request against a
different or stale workspace snapshot fails as a typed workspace error instead
of reading mutable state implicitly.

## NAPI

NAPI uses JSON strings at the binding edge while preserving the shared IDL
shape:

```js
const { Workspace } = require("@omena/napi");

const sources = [{ stylePath: "button.module.css", styleSource: ".button {}" }];
const workspace = new Workspace(process.cwd(), JSON.stringify(sources));
const snapshot = JSON.parse(workspace.snapshotJson());
const diagnostics = JSON.parse(
  workspace.diagnosticsJson(
    JSON.stringify({
      snapshotId: snapshot.snapshotId,
      stylePath: sources[0].stylePath,
      styleSource: sources[0].styleSource,
    }),
  ),
);
```

## WASM

WASM uses in-memory JavaScript values and performs no filesystem access:

```js
import init, { Workspace } from "@omena/wasm";

await init();
const sources = [{ stylePath: "button.module.css", styleSource: ".button {}" }];
const workspace = new Workspace("/workspace", sources);
const snapshot = workspace.snapshot();
const diagnostics = workspace.diagnostics({
  snapshotId: snapshot.snapshotId,
  stylePath: sources[0].stylePath,
  styleSource: sources[0].styleSource,
});
```

## CLI

The process surface accepts one workflow request file and returns a standard CLI
response envelope:

```bash
omena sdk request.json
```

The request contains `workspaceRoot`, `styleSources`, `operation`, and the typed
`request` payload. JSON output wraps the workflow response in
`omena-cli.sdk-workflow` metadata.

## LSP

After `initialize`, send `omena/sdkWorkflow` with `workspaceRoot`, `operation`,
and `request`. The response uses the same public partition and snapshot identity
as diagnostics published for the opened document.

## Responses And Errors

Public responses contain stable workflow output. Debug partitions may include
analysis details and are not a substitute for the public contract. Typed errors
carry a class, code, severity, recoverability, and optional query/input evidence.

The shared classes cover input, workspace, resolution, analysis, transform,
unsupported, internal, and unknown failures. Consumers should branch on typed
fields, not parse message text.

## Lower-Level Bindings

The workflow layer is the cross-surface starting point. Binding-specific and
lower-level query/build calls remain documented in:

- [`omena-napi`](../rust/crates/omena-napi/README.md)
- [`omena-wasm`](../rust/crates/omena-wasm/README.md)
- [CLI commands](../rust/crates/omena-cli/README.md)
- [Rust LSP API](https://docs.rs/omena-lsp-server)
