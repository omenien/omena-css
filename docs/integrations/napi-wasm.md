---
title: NAPI and WASM
description: Select the published Node or browser binding without assuming repository-only exports.
kind: how-to
status: stable
products: [napi, wasm, sdk]
owner: sdk
sourceOfTruth: authored
---

# NAPI and WASM

Choose NAPI when a Node process can load a native package. Choose WASM when the
host needs portable, in-memory execution. Both bindings expose semantic results,
and the coordinated release publishes the same snapshot-bound class surface.

## Published NAPI

`@omena/napi@0.4.0` exports JSON functions and the snapshot-bound `Workspace`
class:

```js
const { checkStyleSourceJson } = require("@omena/napi");

const result = JSON.parse(
  checkStyleSourceJson(".button { color: royalblue; }", "button.module.css"),
);
```

## Published WASM

`@omena/wasm@0.4.0` is a bundler-target package and initializes during import:

```js
import { Workspace } from "@omena/wasm";

const workspace = new Workspace("/workspace", [
  { stylePath: "button.module.css", styleSource: ".button {}" },
]);
const snapshot = workspace.snapshot();
```

WASM performs no filesystem discovery. Supply style sources, package manifests,
and resolution inputs from the host. The [browser playground](../playground.mdx)
demonstrates that constraint directly.

For the complete workflow contract, continue to [SDK workflows](../sdk.md).
