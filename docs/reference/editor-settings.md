---
title: Editor settings
description: Generated ownership and support status for every public VS Code setting.
kind: reference
status: stable
products: [editor, lsp]
owner: editor
sourceOfTruth: generated
---

<!-- Generated from product code. Do not edit by hand. -->

# Editor settings

Each setting has one declared runtime owner. Compatibility-only keys remain visible
so existing workspaces do not fail validation, but the current Rust server ignores
them.

| Setting                                 | Owner                | Status     | Contract                                                                                                                                                                                                                                                      |
| --------------------------------------- | -------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `omena.features.definition`             | Rust language server | Active     | Enable Go to Definition for cx() class names.                                                                                                                                                                                                                 |
| `omena.features.hover`                  | Rust language server | Active     | Enable hover information for cx() class names.                                                                                                                                                                                                                |
| `omena.features.completion`             | Rust language server | Active     | Enable autocomplete inside cx() calls.                                                                                                                                                                                                                        |
| `omena.features.references`             | Rust language server | Active     | Enable Find References for CSS class selectors.                                                                                                                                                                                                               |
| `omena.features.rename`                 | Rust language server | Active     | Enable Rename Symbol for CSS class selectors across SCSS and TS/TSX files.                                                                                                                                                                                    |
| `omena.diagnostics.severity`            | Rust language server | Active     | Severity level for unresolved class diagnostics.                                                                                                                                                                                                              |
| `omena.diagnostics.deepAnalysis`        | Rust language server | Active     | Enable deeper workspace diagnostic analysis in the Rust language server.                                                                                                                                                                                      |
| `omena.diagnostics.unusedSelector`      | Legacy compatibility | Deprecated | Legacy compatibility setting retained for older Node language-server builds. The current Rust server does not consume it.                                                                                                                                     |
| `omena.diagnostics.missingModule`       | Legacy compatibility | Deprecated | Legacy compatibility setting retained for older Node language-server builds. The current Rust server does not consume it.                                                                                                                                     |
| `omena.hover.maxCandidates`             | Legacy compatibility | Deprecated | Legacy compatibility setting retained for older Node language-server builds. The current Rust server does not consume it.                                                                                                                                     |
| `omena.typeFactBackend`                 | VS Code extension    | Active     | Type-fact backend used by the language server. `tsgo` uses the extension-bundled TS 7 native compiler and does not build a synchronous current-TS program on the LSP request path. `tsgo-workspace` uses `pnpm exec tsgo` in the user workspace.              |
| `omena.lspServerRuntime`                | VS Code extension    | Active     | Language-server runtime. `auto` uses the bundled or built Rust `omena-lsp-server` and fails fast if no binary is available. `omena-lsp-server` requires the bundled binary, explicit `OMENA_LSP_SERVER_PATH`, or explicit `OMENA_LSP_SERVER_COMMAND` command. |
| `omena.pathAlias`                       | Legacy compatibility | Deprecated | Legacy compatibility map for older Node language-server builds. The current Rust server does not consume this setting; configure package manifests through `omena.resolution.packageManifestPaths` where applicable.                                          |
| `omena.scss.classnameTransform`         | Legacy compatibility | Deprecated | Legacy compatibility mirror of css-loader's `modules.localsConvention` for older Node language-server builds. The current Rust server does not consume this setting.                                                                                          |
| `omena.resolution.packageManifestPaths` | Rust language server | Active     | Package manifest file paths or file URIs loaded by the Rust language server for Sass package resolution.                                                                                                                                                      |
