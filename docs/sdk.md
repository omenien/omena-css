---
title: SDK workflows
description: Use snapshot-bound Omena workflows from NAPI, WASM, CLI, and LSP clients.
kind: how-to
status: stable
products: [sdk, napi, wasm, cli, lsp]
owner: sdk
sourceOfTruth: hybrid
---

# SDK Workflows

omena-css exposes the same snapshot-bound workflows through NAPI, WASM, the
`omena` process, and the Rust LSP. The IDL in
`contracts/engine-sdk-workflow/main.tsp` owns request, response, partition, and
typed error envelopes.

For CSS Modules, the emitted token is not a contract; `classMap`, `namedExports`,
and the generated `.d.ts` are. Hand-writing an emitted token into markup, tests,
or CSS is unsupported.

## Registry Availability

The repository and npm registry do not currently expose identical NAPI
versions. Choose examples by the version you can install, not by the repository
source alone.

| Package       | Latest published | API shape                                     |
| ------------- | ---------------- | --------------------------------------------- |
| `@omena/napi` | `0.2.1`          | JSON functions such as `checkStyleSourceJson` |
| `@omena/wasm` | `0.3.0`          | JavaScript values and `Workspace`             |

The Node `Workspace` implementation in this repository belongs to the
unreleased NAPI `0.3.0` surface.

## Install

```bash
npm install @omena/napi@0.2.1
npm install @omena/wasm@0.3.0
cargo install omena-cli --locked
```

## Workflow Matrix

This table is generated from the TypeSpec request models and checked against the
repository's four-surface parity matrix. It describes source-level contract
coverage; it is not a registry-availability claim.

<!-- BEGIN GENERATED: OMENA SDK WORKFLOWS -->
<!-- Generated from product code. Do not edit by hand. -->

| Workflow      | Covered surfaces             |
| ------------- | ---------------------------- |
| `snapshot`    | `napi`, `wasm`, `cli`, `lsp` |
| `query`       | `napi`, `wasm`, `cli`, `lsp` |
| `diagnostics` | `napi`, `wasm`, `cli`, `lsp` |
| `build`       | `napi`, `wasm`, `cli`, `lsp` |
| `explain`     | `napi`, `wasm`, `cli`, `lsp` |

<!-- END GENERATED: OMENA SDK WORKFLOWS -->

Every operation after `snapshot` carries its `snapshotId`. A request against a
different or stale workspace snapshot fails as a typed workspace error instead
of reading mutable state implicitly.

## NAPI

The published NAPI package uses JSON strings at the binding edge:

```js
const { checkStyleSourceJson } = require("@omena/napi");

const report = JSON.parse(
  checkStyleSourceJson(".button { color: royalblue; }", "button.module.css"),
);
```

Repository source also defines a snapshot-bound Node `Workspace`, but that class
is not part of `@omena/napi@0.2.1`. Do not copy the workspace example into a
published-package integration until a compatible NAPI release is available.

## WASM

The published WASM package is a bundler-target module. Importing it starts the
module; there is no default `init()` export. It uses in-memory JavaScript values
and performs no filesystem access:

```js
import { Workspace } from "@omena/wasm";

const sources = [{ stylePath: "button.module.css", styleSource: ".button {}" }];
const workspace = new Workspace("/workspace", sources);
const snapshot = workspace.snapshot();
const diagnostics = workspace.diagnostics({
  snapshotId: snapshot.snapshotId,
  stylePath: sources[0].stylePath,
  styleSource: sources[0].styleSource,
});
```

The [browser playground](playground.mdx) loads a web-target build of this same
crate and demonstrates workspace diagnostics, semantic transforms, and
target-aware builds without uploading source. Its examples use caller-supplied
in-memory files and deliberately omit filesystem discovery and host-specific
workspace resolution.

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
