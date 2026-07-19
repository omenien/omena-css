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

Provenance is verified by CLI/CI workflows, never by latency-sensitive LSP
requests. The language server reads local source, SIF, and lockfile data but does
not fetch registry metadata or transparency logs while serving editor requests.

| Tier | Required evidence                                                                         |
| ---- | ----------------------------------------------------------------------------------------- |
| T0   | No enforced provenance verification is available for the selected entry.                  |
| T1   | Local lockfile and SIF integrity verification.                                            |
| T2   | Verified package or third-party attestation; a recorded reference alone remains advisory. |
| T3   | Verified omena-toolchain attestation whose signed subject is the selected SIF artifact.   |

Record npm provenance metadata from a local JSON response:

```sh
omena lock fetch-provenance design-system \
  --lockfile omena.lock \
  --npm-metadata npm-metadata.json \
  --json
```

Verify a Sigstore bundle locally and bind the result to the lock entry:

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

Offline verifier reports can be recorded without network access:

```sh
omena lock record-verification design-system \
  --lockfile omena.lock \
  --verification attestation-verification.json \
  --json
```

For T3 evidence, also pass the matching SIF artifact so omena can compare the
signed `sha256` subject digest with the selected entry's `sifPath` bytes:

```sh
omena lock record-verification design-system \
  --lockfile omena.lock \
  --verification t3-attestation-verification.json \
  --artifact sif/design-system.sif.json \
  --json
```

Enforce the required tier in CI:

```sh
omena lock verify --lockfile omena.lock --tier t2 --frozen --json
omena lock verify --lockfile omena.lock --tier t3 --frozen --json
```
