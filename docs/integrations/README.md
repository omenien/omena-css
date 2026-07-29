---
title: Integrations
description: Choose the Omena host surface that matches an editor, build tool, Node process, or browser.
kind: how-to
status: stable
products: [sdk, bundler, lint]
owner: developer-experience
sourceOfTruth: authored
---

# Integrations

Omena exposes one semantic engine through several hosts. The hosts differ in
filesystem access, lifecycle, and publication status.

| Host | Best for | Filesystem model | Availability |
| --- | --- | --- | --- |
| CLI | scripts and CI | process filesystem | `omena-cli@0.3.0` on crates.io |
| Rust LSP | editors | editor workspace | GitHub and extension bundles |
| NAPI | Node hosts | caller and native process | `@omena/napi@0.2.1` |
| WASM | browsers and portable hosts | caller-supplied memory | `@omena/wasm@0.3.0` |
| Vite | development and builds | Vite module graph | `@omena/vite-plugin@0.2.1` |
| PostCSS | PostCSS pipelines | PostCSS file context | `@omena/postcss-plugin@0.2.1` |

Use the [SDK workflow guide](../sdk.md) for request and response contracts.
Build-tool pages focus on host-specific lifecycle and option boundaries.

- [NAPI and WASM](./napi-wasm.md)
- [Vite](./vite.md)
- [PostCSS](./postcss.md)
- [Lint integrations](./linting.md)
