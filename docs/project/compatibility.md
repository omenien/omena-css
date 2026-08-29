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

| Surface                            | Public version | Status                                     |
| ---------------------------------- | -------------- | ------------------------------------------ |
| CLI and Rust crate train           | `0.5.0`        | CLI archives; coordinated 51-crate train   |
| `omena-reactive`                   | `0.5.0`        | first publication in the coordinated train |
| `@omena/wasm`                      | `0.5.0`        | release-managed npm binding                |
| `@omena/napi`                      | `0.5.0`        | native binding and `Workspace` class       |
| `@omena/css-build-adapter`         | `0.5.0`        | release-managed integration                |
| `@omena/vite-plugin`               | `0.5.0`        | release-managed integration                |
| `@omena/postcss-plugin`            | `0.5.0`        | release-managed integration                |
| ESLint, Stylelint, Oxlint adapters | none           | repository-only                            |
| VS Code extension                  | `5.4.0`        | coordinated VSIX release                   |
| Legacy VS Marketplace listing      | `5.2.0`        | older publisher listing                    |

Repository source can be newer than this table. Installation pages pin public
versions and call out source-only APIs. Release updates must change this page
from registry evidence, not from workspace package versions alone.
