---
title: Compatibility and availability
description: Check which Omena versions and host surfaces are currently available from public registries.
kind: reference
status: stable
products: [release, sdk, editor, bundler]
owner: release
sourceOfTruth: authored
---

# Compatibility and availability

This page records public availability, not merely source present in the
repository.

| Surface                            | Public version | Status                                                 |
| ---------------------------------- | -------------- | ------------------------------------------------------ |
| CLI and Rust crate train           | `0.3.0`        | CLI archives; 50 of 51 publishable crates on crates.io |
| `omena-reactive`                   | none           | repository-only                                        |
| `@omena/wasm`                      | `0.3.0`        | published                                              |
| `@omena/napi`                      | `0.2.1`        | published; no Node `Workspace` class                   |
| `@omena/css-build-adapter`         | `0.2.1`        | published                                              |
| `@omena/vite-plugin`               | `0.2.1`        | published                                              |
| `@omena/postcss-plugin`            | `0.2.1`        | published                                              |
| ESLint, Stylelint, Oxlint adapters | none           | repository-only                                        |
| VS Code extension                  | `5.3.0`        | GitHub VSIX and Open VSX                               |
| Legacy VS Marketplace listing      | `5.2.0`        | older publisher listing                                |

Repository source can be newer than this table. Installation pages pin public
versions and call out source-only APIs. Release updates must change this page
from registry evidence, not from workspace package versions alone.
