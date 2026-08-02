---
title: CSS Modules token rotation
description: Compatibility notice for an upcoming CSS Modules token identity rotation.
kind: explanation
status: preview
products: [cli, sdk, napi, wasm, lsp]
owner: release
sourceOfTruth: authored
---

# CSS Modules Token Rotation

The release version and rotation identifier are pending.

The emitted token is not a contract; `classMap`, `namedExports`, and the
generated `.d.ts` are. Hand-writing an emitted token into markup, tests, or CSS
is unsupported. Consumers should read the generated interface rather than
persisting or constructing emitted names.
