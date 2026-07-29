---
title: LSP capabilities
description: The generated language-server capability contract reported during initialization.
kind: reference
status: stable
products: [lsp, editor]
owner: lsp
sourceOfTruth: generated
---

<!-- Generated from product code. Do not edit by hand. -->

# LSP capabilities

The table is rendered from the Rust server's serialized initialize capability contract.

| Capability path                                  | Value                                               |
| ------------------------------------------------ | --------------------------------------------------- |
| `textDocumentSync`                               | `2`                                                 |
| `definitionProvider`                             | `true`                                              |
| `hoverProvider`                                  | `true`                                              |
| `colorProvider`                                  | `true`                                              |
| `completionProvider.triggerCharacters`           | `["'","\"","`",",",".","$","@","-"]`                |
| `completionProvider.resolveProvider`             | `false`                                             |
| `codeActionProvider.codeActionKinds`             | `["quickfix","refactor.extract","refactor.inline"]` |
| `codeActionProvider.resolveProvider`             | `false`                                             |
| `referencesProvider`                             | `true`                                              |
| `codeLensProvider.resolveProvider`               | `false`                                             |
| `documentLinkProvider.resolveProvider`           | `false`                                             |
| `workspaceSymbolProvider`                        | `true`                                              |
| `renameProvider.prepareProvider`                 | `true`                                              |
| `workspace.workspaceFolders.supported`           | `true`                                              |
| `workspace.workspaceFolders.changeNotifications` | `true`                                              |
