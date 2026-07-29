# omena-sif

## Role

`omena-sif` owns Sass and Less interface artifacts, lockfile records, canonical
serialization, fingerprint chains, provenance records, and structural diffs.

## Contract

The primary types are `OmenaSifV1`, `OmenaLifExportsV1`, and `OmenaLockV1`.
Static generators extract public exports, deterministic writers preserve the
schema, and BLAKE3 fingerprints bind source, interface, transitive dependencies,
and generator identity.

## Consumers

The CLI uses this crate for `sif`, `lock`, and provenance commands. Query, LSP,
and bridge layers consume the same artifacts for external Sass and Less
resolution.

## Boundaries

This crate does not evaluate Sass or Less, execute packages, traverse the
filesystem, or access the network. Fetching artifacts and cryptographically
verifying Sigstore bundles remain host responsibilities.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-sif
```
