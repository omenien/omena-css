---
title: Release process
description: The normalized rules and order for every Omena release channel.
kind: reference
status: stable
products: [release, editor, cli, crates, npm]
owner: release
sourceOfTruth: authored
---

# Release process

Every Omena release follows one normalized process. It is contract-enforced by
`release/release/notes` (`scripts/release-notes.ts check`), the publish-train
closure gate, and the provenance source guard — not by operator memory. The
shape is deliberately close to high-discipline OSS trains (oxc's
prepare-release → human approval → version-diff-triggered publish), adapted to
this repository's stronger primitives: a machine-declared semver-intent
contract, immutable-tag publication, and post-publication body verification.

## Tag families

| family        | channel                                        | example          |
| ------------- | ---------------------------------------------- | ---------------- |
| `release-v*`  | Rust crate train, CLI binaries, npm bindings   | `release-v0.4.0` |
| `vscode-v*`   | Editor extension (Marketplace + Open VSX)      | `vscode-v5.4.0`  |
| `v*` (legacy) | Pre-rebrand `css-module-explainer` era, frozen | `v4.1.19`        |

Rules: one tag per family per version; tags are immutable and are the only
publication sources; release names never repeat the tag string; both active
families are registered in `docs/releases/manifest.json` before their
workflows run (the notes gate fails otherwise).

## Release order

1. **Verify before printing** — the full release verifier
   (`release/release/verify`) must be green at the pre-version pin.
2. **Print versions** — one commit moves the workspace, extension, npm, and
   internal exact pins together and registers the release pages/manifest
   entries. Until a train departs, `master` keeps the last published version
   string; breaking work in flight is declared in the release-semver intent
   contract (baseline → target), never by an early version bump.
3. **Tag** — `release-v<version>` on the verified commit. The tag is the
   immutable boundary; everything after it is publication, not development.
4. **Dry-run first** — every publish workflow runs its `dry_run` path before
   the real dispatch (the crate train additionally re-runs the steady-state
   semver gate against the live registry).
5. **Publish from the tag** — crate train, CLI, npm, and extension workflows
   are dispatched **on the tag ref**, never from a branch head; partial
   failures resume with the `resume` input (registries reject duplicates, so
   reruns are safe).
6. **Verify after publishing** — registry state, release bodies
   (`verify-github`), attestations, and the same-pin push CI must all be
   green before the release is closed out.
7. **Rotate the compatibility window** — after live publication is verified,
   append the immutable Rust release tag to
   `rust/omena-published-release-baselines.json`, move the active semver-intent
   baseline to that version, and open the next `0.x` minor window with an empty
   intent set. The release-semver intent gate binds these two records and rejects
   a completed train left active after publication.

## Provenance rule

The dispatch ref and the publication checkout must be the same commit. Both
publish workflows carry an inline guard immediately after checkout that
hard-fails before any publish step when `github.sha` differs from the
checked-out `HEAD` — dispatching a tag build from a moving branch silently
poisons the signed SLSA/attestation source metadata (learned irreversibly in
the 0.4.0 train; nine npm packages and the VSIX permanently attest a commit
that was not built). Dispatch release workflows from the tag ref itself.

## Release notes pipeline

- The source of truth is the authored release page in `docs/releases/` (the
  docs-site frontmatter is stripped at render time and never appears in a
  release body).
- The rendered body is: authored highlights → `## Changes` (generated:
  declared breaking changes from the semver-intent contract, then
  Features/Fixes/Performance from conventional-commit subjects over the
  compare range) → `## Distribution` (channel-specific facts from the
  manifest) → `## Release links` (source, changelog, full diff).
- Workflows render with `--changelog`, publish with `body_path`, and then
  `verify-github` asserts the persisted body matches byte-for-byte.
- GitHub's `generate_release_notes` is never used for registered releases.

## Historical releases

The 60 legacy `v*` releases are frozen history: bodies carry a pre-rebrand
banner (and a pointer when a duplicate exists under the current families),
assets are never modified, and tags are never deleted. The
`release:notes backfill` command is the only sanctioned editor for published
bodies; it is idempotent and non-destructive.
