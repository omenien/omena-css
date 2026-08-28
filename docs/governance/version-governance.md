---
title: Version governance
description: The independently versioned extension, Rust crate train, and npm release contracts.
kind: reference
status: stable
products: [release, extension, crates, npm]
owner: release
sourceOfTruth: hybrid
---

# Version governance

Omena uses independent version axes so an editor release cannot accidentally
publish or retag the Rust crate train. The table below is checked against the
authoritative manifests and release gates.

## Derived contract

| Key                            | Value                                                                                                                                                                | Authority                                                   |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `extensionVersion`             | `5.4.0`                                                                                                                                                              | root `package.json`                                         |
| `extensionVersionLine`         | `5.x`                                                                                                                                                                | root `package.json` major                                   |
| `crateTrainVersion`            | `0.4.0`                                                                                                                                                              | `rust/Cargo.toml` workspace package                         |
| `crateTrainVersionLine`        | `0.x`                                                                                                                                                                | Rust workspace major                                        |
| `crateTrainTagPrefix`          | `release-v`                                                                                                                                                          | release tag grammar gate                                    |
| `extensionTagPrefix`           | `vscode-v`                                                                                                                                                           | release tag grammar gate                                    |
| `linkedEmissionTargetRelease`  | `0.5.0`                                                                                                                                                              | linked-emission default-surfaces contract                   |
| `changesetIgnoredPackages`     | `@omena/check-orchestrator, @omena/checker, @omena/eslint-plugin, @omena/oxlint-plugin, @omena/stylelint-plugin, @omena/vite-plugin, @omena/vitest, @omena/examples` | `.changeset/config.json`                                    |
| `separateFirstPublishPackages` | `@omena/css-build-adapter, @omena/postcss-plugin`                                                                                                                    | private package manifests outside the Changesets ignore set |
| `releaseManagedNpmBindings`    | `@omena/napi, @omena/napi-*, @omena/wasm`                                                                                                                            | npm publish workflow                                        |

## Independent axes

The extension version comes from the root manifest and uses `vscode-vX.Y.Z`
tags. The Rust workspace version is shared by publishable crates and generated
NAPI/WASM manifests and uses `release-vX.Y.Z` tags. Exact Rust inter-crate pins
must equal the workspace version. A change to one axis does not imply a change
to the other unless a coordinated release explicitly moves both.

## Reserved majors

Linked-order emission is the admitted default for the `0.5.0` Rust train after
full-corpus differential coverage and a zero unexpected-divergence census were
established. The successor surface contract keeps Rust options, CLI, N-API,
WASM, the JavaScript adapter, and Vite on that default. The `release-v0.5.0`
train must not publish before the linked-emission closure pin exists.

Rust crate `1.0.0` remains reserved until a release proposal cites a train-wide
public API freeze artifact.

The current query and bundler snapshots at
`rust/crates/omena-query/tests/snapshots/public-api.txt` and
`rust/crates/omena-bundler/tests/snapshots/public-api.txt` detect local API
drift, but they are not a train-wide freeze declaration. Until that broader
artifact and review exist, published crates stay on the `0.x` line.

## Pre-1.0 breaking changes

On the Rust `0.x` line, a breaking contract change increments the minor version.
Patch versions are reserved for compatible fixes. The staged `0.2.1` workspace
demonstrated why this distinction matters: a generator identity rotation and
expanded public surfaces cannot be released as a patch. The current `0.4.0`
train likewise advances the minor line for reviewed pre-1.0 public-shape and
runtime-contract changes.

## Publish status

| Surface                                             | Release policy                                                                                                       |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Publishable Rust crates                             | One exact workspace version and one dependency-ordered `release-vX.Y.Z` train.                                       |
| Never-published Rust crate names                    | Explicit first-publish members of the same train; they select protected bootstrap authentication until registered.   |
| `@omena/wasm`, `@omena/napi`, `@omena/napi-*`       | Generated from the Rust workspace version and selected explicitly in the npm publish workflow.                       |
| Changeset-ignored tooling packages                  | Excluded from this coordinated train; their package-local `0.0.x` versions do not follow the root extension version. |
| `@omena/css-build-adapter`, `@omena/postcss-plugin` | Private, package-local `0.0.x` surfaces. Their first public publish requires a separate decision.                    |

Registry publication is non-atomic and irreversible. A version commit and a
successful dry run are preparation evidence, not registry publication evidence;
operators follow [the release runbook](../../RELEASING.md) for channel-specific
checks and recovery.
