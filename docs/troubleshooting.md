---
title: Troubleshooting
description: Diagnose installation, language-server, workspace-resolution, and build-integration failures.
kind: how-to
status: stable
products: [cli, editor, lsp, sdk]
owner: developer-experience
sourceOfTruth: authored
---

# Troubleshooting

Start with the surface that failed. A green CLI command does not prove that an
editor found its server binary, and a repository build does not prove that the
same API is published to npm.

## The editor does not start

1. Open **Omena CSS Modules: Show Language Server Output**.
2. Check `omena.lspServerRuntime`. `auto` still requires a bundled binary,
   `OMENA_LSP_SERVER_PATH`, or `OMENA_LSP_SERVER_COMMAND`.
3. Run the binary directly:

   ```sh
   omena-lsp-server --help
   ```

4. Check the [editor settings reference](./reference/editor-settings.md). Keys
   marked deprecated are accepted by VS Code but ignored by the Rust server.

## An import resolves in one host but not another

Resolution inputs belong to the host. The CLI, editor, Vite, and browser WASM
do not discover identical filesystem state.

- For the Rust LSP, provide package manifests through
  `omena.resolution.packageManifestPaths`.
- For build plugins, pass `sources` and `packageManifests` when the transform
  depends on files outside the current module.
- For WASM, provide all files and resolution mappings in memory. Browser code
  cannot scan the workspace.

## An npm example has no exported symbol

Compare the installed version with the
[compatibility matrix](./project/compatibility.md). The repository contains
APIs that may not have reached npm. In particular, `Workspace` is not exported
by `@omena/napi@0.2.1`.

```sh
npm view @omena/napi version
npm view @omena/wasm version
```

## A transform is skipped

Omena keeps transformations conservative when required facts are absent. Use
`omena explain`, request build evidence, or inspect the structured result
instead of assuming that an unchanged file means the pass was not invoked.

## Report a reproducible problem

Include the Omena version, surface, source syntax, smallest input, command or
setting, actual result, and expected result. See
[support](./project/support.md) for security and issue-reporting routes.
