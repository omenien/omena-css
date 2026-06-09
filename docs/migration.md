# External Sass Module Migration

This document describes the compatibility path from legacy external-reference
handling to SIF-backed external Sass module analysis.

## Current Default

In the current v5.2 line, `omena style-diagnostics` enables SIF discovery when
`--external` is omitted. This means bare or aliased external Sass references are
reported through the external boundary diagnostic surface even before a lockfile
has been authored.

The SIF-aware path is also enabled by either:

- Passing `--external sif`.
- Passing `--lockfile <path>`.
- Placing `omena.lock` in the target file's directory or one of its ancestors.

Passing `--external ignored` is an explicit compatibility opt-out. It keeps the
legacy boundary behavior even when `omena.lock` exists.

## Phase 0: Compatibility Mode

Phase 0 remains available as the explicit compatibility opt-out.

```sh
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

## Phase 2: Default SIF Discovery

Phase 2 is the current default behavior:

- Omitted `--external` enables SIF discovery by default.
- `--external ignored` remains the explicit compatibility escape hatch.
- Compatibility gates cover omitted mode, explicit ignored mode, explicit SIF
  mode, lockfile-triggered mode, malformed lockfile diagnostics, and resolved
  SIF-backed packages.

## Compatibility Matrix

| Invocation                                                        | `omena.lock`          | Expected behavior                                    |
| ----------------------------------------------------------------- | --------------------- | ---------------------------------------------------- |
| `style-diagnostics app.module.scss`                               | absent                | Phase 2 SIF discovery; external boundary diagnostics |
| `style-diagnostics app.module.scss --external ignored`            | absent or present     | Phase 0 compatibility; reversible opt-out            |
| `style-diagnostics app.module.scss --external sif`                | absent                | SIF boundary diagnostics enabled                     |
| `style-diagnostics app.module.scss --lockfile path/to/omena.lock` | explicit path         | SIF boundary diagnostics enabled                     |
| `style-diagnostics app.module.scss`                               | present and valid     | Lockfile-triggered SIF mode                          |
| `style-diagnostics app.module.scss`                               | present but malformed | `lockfileInvalid` diagnostic                         |

## Lockfile Version Compatibility

`omena.lock` is deterministic JSON. The current schema includes
`lockfileVersion`, `entries`, and optional `omenaMinVersion`. `lock verify
--frozen` fails when a lockfile requires a future omena runtime.

Older omena versions that do not understand SIF mode ignore `omena.lock` as
ordinary workspace data. Newer versions must keep `--external ignored` as the
documented compatibility escape hatch for the migration window.

## Provenance Verification

SIF provenance is enforced through the CLI and lockfile, not through LSP request
handling. The language server reads local workspace files, SIF artifacts, and
`omena.lock`; it never fetches registry metadata or queries transparency logs
while serving editor requests.

Recorded provenance references are advisory until verified evidence is added.
For npm provenance metadata, first record the reference:

```sh
omena lock fetch-provenance design-system \
  --lockfile omena.lock \
  --npm-metadata npm-metadata.json \
  --json
```

Then record verified evidence through one of the local verification paths:

```sh
omena lock verify-attestation design-system \
  --lockfile omena.lock \
  --artifact package.tgz \
  --bundle package.sigstore.json \
  --reference https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance \
  --kind npm-provenance.sigstore \
  --verified-tier t2 \
  --issuer https://token.actions.githubusercontent.com \
  --statement-type https://in-toto.io/Statement/v1 \
  --statement-predicate-type https://slsa.dev/provenance/v1
```

Offline verifier reports can also be recorded:

```sh
omena lock record-verification design-system \
  --lockfile omena.lock \
  --verification attestation-verification.json \
  --json
```

For T3 omena-toolchain evidence, the verified subject is the SIF artifact
itself. Both direct verification and offline report ingestion must bind the
signed provenance statement to the selected lock entry's `sifPath`; offline
reports also require the matching SIF JSON artifact so omena can check the
statement's `sha256` subject digest against the artifact bytes:

```sh
omena lock record-verification design-system \
  --lockfile omena.lock \
  --verification t3-attestation-verification.json \
  --artifact sif/design-system.sif.json \
  --json
```

Finally, CI can enforce the required tier:

```sh
omena lock verify --lockfile omena.lock --tier t2 --frozen --json
omena lock verify --lockfile omena.lock --tier t3 --frozen --json
```
