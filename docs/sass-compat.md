---
title: External Sass and SIF compatibility
description: Configure external Sass interfaces, lockfiles, and provenance-aware compatibility modes.
kind: how-to
status: stable
products: [cli, sass, sif]
owner: sass
sourceOfTruth: authored
---

# External Sass And SIF Compatibility

This guide describes how external Sass references, SIF artifacts, and
`omena.lock` interact. Source codemods are a separate plan-first workflow; see
[Using `omena migrate`](migrate-verb.md).

## Compatibility Matrix

The CLI discovers SIF-backed external modules by default. Every supported mode
and escape hatch is summarized once here:

| `style-diagnostics` invocation                         | Lockfile state       | Behavior                                                         |
| ------------------------------------------------------ | -------------------- | ---------------------------------------------------------------- |
| `app.module.scss --json`                               | absent               | SIF discovery; unresolved external references become diagnostics |
| `app.module.scss --external ignored --json`            | absent or present    | Compatibility opt-out; external-boundary diagnostics are skipped |
| `app.module.scss --external sif --json`                | absent               | SIF boundary diagnostics are explicitly enabled                  |
| `app.module.scss --lockfile path/to/omena.lock --json` | explicit valid path  | The selected lockfile supplies SIF entries                       |
| `app.module.scss --json`                               | discovered and valid | The nearest ancestor `omena.lock` supplies SIF entries           |
| `app.module.scss --json`                               | discovered malformed | A `lockfileInvalid` diagnostic is returned                       |

`--external ignored` is the reversible compatibility escape hatch.
It remains effective even when a populated lockfile would otherwise resolve an
external package.

## Adopt SIF-Backed Resolution

Generate a SIF artifact, record it in the lockfile, then run diagnostics without
an external-mode flag:

```sh
omena sif generate tokens.scss \
  --canonical-url design-system/tokens \
  --output tokens.sif.json

omena lock update --lockfile omena.lock --sif tokens.sif.json --json

omena style-diagnostics app.module.scss --json
```

When the canonical URL matches a Sass reference, diagnostics resolve exports
through the selected SIF. Missing, partial, and stale interfaces remain explicit
boundary outcomes rather than silently falling back to network access.

Malformed or unreadable lockfiles are reported through the normal JSON
diagnostic envelope. They do not abort before style diagnostics are produced.

## Lockfile Contract

`omena.lock` is deterministic camelCase JSON. Its top-level schema contains:

- `lockfileVersion`: required wire-format version.
- `entries`: required, canonically sorted SIF entry array.
- `omenaMinVersion`: optional minimum compatible omena runtime.

`omena lock verify --frozen` rejects drift and a lockfile requiring a future
runtime. Older tools that do not implement SIF treat `omena.lock` as workspace
data; current tools preserve `--external ignored` for explicit compatibility.

## Provenance Verification

Provenance is acquired and recorded by CLI/CI workflows, never by
latency-sensitive LSP requests. For an external-SIF cache shard above T1, the
native bridge verifies the recorded Sigstore bundle offline at consumption time
against the production Fulcio root, Rekor inclusion proof, OIDC issuer, and
approved workflow identity. The language server does not fetch registry metadata
or transparency logs while serving editor requests.

| Tier | Required evidence                                                                                |
| ---- | ------------------------------------------------------------------------------------------------ |
| T0   | No enforced provenance verification is available for the selected entry.                         |
| T1   | Local lockfile and SIF integrity verification, including third-party provenance metadata.        |
| T2   | An Omena CI identity signed the canonical URL, tier, and SIF hash in a published subject.        |
| T3   | The same Omena-published subject binding under the stricter release-workflow provenance posture. |

Every external-SIF cache hit is compared with SIF bytes regenerated from the
current local source before it can be served. A cache entry is discarded and
regenerated when that comparison is unavailable or differs, including when no
recorded verdict exists or a local verdict file has been deleted. Consequently,
pure external SIF bytes with no locally readable source are not a servable cache
path.

T2 and T3 are advisory provenance labels for Omena-published artifacts only.
`lock verify-attestation` records a verdict for the exact canonical URL, tier,
and SIF hash, plus a content-addressed Sigstore bundle whose signed subject binds
all three values. Bridge reconstructs that subject and verifies the bundle
offline at consumption time; LSP consumes the resulting label without reading a
lockfile or using the network. Missing, forged, mismatched, or third-party
evidence remains at T0/T1. No tier widens a cache partition, permits
cross-workspace serving, or enables a product capability. Wasm builds contain no
verifier and therefore never attach Omena-published T2/T3 provenance.

Published subjects do not expire by wall-clock time, and serving performs no
online revocation lookup. Freshness instead follows the locally regenerated SIF
hash on every serve: a source change produces a new hash, so an older subject
and verdict cannot authenticate the replacement bytes. A previously recorded
bundle remains valid only for its exact URL, tier, and hash until the local
verdict is removed or the verifier's trusted-root policy changes. Removing the
local verdict drops the elevated label but never enables cached bytes to bypass
regeneration.

Acquire npm registry metadata through the platform npm CLI, outside the Omena
binary, and record a deterministic present/absent receipt beside it:

```sh
pnpm acquire:sif-npm-provenance design-system@1.0.0 \
  --output npm-metadata.json \
  --receipt npm-metadata.receipt.json
```

The acquisition receipt names `platform-npm-cli` as the network owner. Omena
does not fetch registry metadata: `lock fetch-provenance` only ingests the local
JSON file after validating its package, version, provenance shape, and
attestation subject. A receipt with `provenanceDisposition: "absent"` records
the absence without upgrading the lock entry above T1. Native registry fetching
inside Omena remains deferred.

For deterministic or offline automation, `--metadata-file response.json`
replaces `npm view` while preserving the same output and receipt shape. Feed the
resulting local metadata into the lock command:

```sh
omena lock fetch-provenance design-system \
  --lockfile omena.lock \
  --npm-metadata npm-metadata.json \
  --json
```

Local verifier reports may still be recorded as non-elevating T0/T1 metadata:

```sh
omena lock record-verification design-system \
  --lockfile omena.lock \
  --verification attestation-verification.json \
  --json
```

Local report JSON and third-party attestations cannot establish T2 or T3. For an
Omena-published SIF label, use `verify-attestation` with the matching canonical
SIF and the keyless bundle for its `*.attestation-subject.json`. The CLI
reconstructs that subject, verifies it, and records both a content-addressed
bundle and a URL/tier/hash-bound verdict:

```sh
omena lock verify-attestation design-system \
  --lockfile omena.lock \
  --artifact sif/design-system.sif.json \
  --bundle sif/design-system.sigstore.json \
  --reference github-attestation:RUN_ID \
  --kind omena-toolchain.sigstore \
  --verified-tier t3 \
  --identity https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master \
  --issuer https://token.actions.githubusercontent.com \
  --json
```

Enforce the required tier in CI:

```sh
omena lock verify --lockfile omena.lock --tier t2 --frozen --json
omena lock verify --lockfile omena.lock --tier t3 --frozen --json
```
