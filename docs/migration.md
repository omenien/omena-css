# External Sass Module Migration

This document describes the compatibility path from legacy external-reference
handling to SIF-backed external Sass module analysis.

## Current Default

In the current v5.2 line, `omena style-diagnostics` preserves the legacy
compatibility default when no lockfile is present and `--external` is omitted.
That mode does not enable external SIF boundary diagnostics.

The SIF-aware path is enabled by either:

- Passing `--external sif`.
- Passing `--lockfile <path>`.
- Placing `omena.lock` in the target file's directory or one of its ancestors.

Passing `--external ignored` is an explicit compatibility opt-out. It keeps the
legacy boundary behavior even when `omena.lock` exists.

## Phase 0: Compatibility Mode

Phase 0 is the default for workspaces without `omena.lock`.

```sh
omena style-diagnostics app.module.scss --json
omena style-diagnostics app.module.scss --external ignored --json
```

This mode is useful when a workspace has not adopted SIF artifacts yet, or when
CI needs a reversible way to keep the previous external-reference behavior.

## Phase 1: Lockfile-Triggered SIF Mode

Phase 1 starts when the workspace opts into SIF analysis.

```sh
omena sif generate tokens.scss \
  --canonical-url design-system/tokens \
  --output tokens.sif.json

omena lock update --lockfile omena.lock --sif tokens.sif.json --json

omena style-diagnostics app.module.scss --json
```

When `omena.lock` is discovered, `style-diagnostics` reads the lockfile entries
and enables SIF-backed boundary diagnostics. External references backed by a
matching SIF can resolve Sass exports through the external module path.

Malformed or unreadable lockfiles are reported through the normal JSON
diagnostic surface with `code: "lockfileInvalid"`. They do not abort before
style diagnostics can be produced.

## Phase 2: Planned Default SIF Discovery

Phase 2 is not the current default. The planned behavior is:

- Omitted `--external` enables SIF discovery by default.
- `--external ignored` remains the explicit compatibility escape hatch.
- `omena.lock` generation and update behavior is documented before the flip.
- Compatibility gates cover omitted mode, explicit ignored mode, explicit SIF
  mode, lockfile-triggered mode, malformed lockfile diagnostics, and resolved
  SIF-backed packages.

Until Phase 2 is explicitly released, do not assume a workspace without
`omena.lock` is analyzed with external SIF boundary diagnostics.

## Compatibility Matrix

| Invocation                                                        | `omena.lock`          | Expected behavior                                       |
| ----------------------------------------------------------------- | --------------------- | ------------------------------------------------------- |
| `style-diagnostics app.module.scss`                               | absent                | Phase 0 compatibility; no external boundary diagnostics |
| `style-diagnostics app.module.scss --external ignored`            | absent or present     | Phase 0 compatibility; reversible opt-out               |
| `style-diagnostics app.module.scss --external sif`                | absent                | SIF boundary diagnostics enabled                        |
| `style-diagnostics app.module.scss --lockfile path/to/omena.lock` | explicit path         | SIF boundary diagnostics enabled                        |
| `style-diagnostics app.module.scss`                               | present and valid     | Lockfile-triggered SIF mode                             |
| `style-diagnostics app.module.scss`                               | present but malformed | `lockfileInvalid` diagnostic                            |

## Lockfile Version Compatibility

`omena.lock` is deterministic JSON. The current schema includes
`lockfileVersion`, `entries`, and optional `omenaMinVersion`. `lock verify
--frozen` fails when a lockfile requires a future omena runtime.

Older omena versions that do not understand SIF mode ignore `omena.lock` as
ordinary workspace data. Newer versions must keep `--external ignored` as the
documented compatibility escape hatch for the migration window.
