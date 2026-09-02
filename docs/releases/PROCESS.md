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

Rules: one tag per family per version; tags are immutable source selectors, but
pushing them starts no publication workflow; release names never repeat the tag string; both active
families are registered in `docs/releases/manifest.json` before their
workflows run (the notes gate fails otherwise).

The no-trigger rule is ancestry-scoped: a tag that points to a commit predating
this policy resolves that commit's older workflow files. Historical rebuilds
therefore use explicit dispatch; never create a probe tag to test this rule.
Dependency and Dependabot housekeeping is not release-train work: land it before
the preflight window or after closeout, not between print and publication.

## Release order

1. **Verify before printing** — the full release verifier
   (`release/release/verify`) must be green at the pre-version pin. The preflight
   note must also cite a lane-green Release Rehearsal run from the default branch
   no older than 14 days; this reviewer-checked receipt is not a push-tier gate.
2. **Print versions with their evidence** — stage the whole print first, then
   run `pnpm check:rust-domain-claim-census -- --write` so its index-backed
   census sees every tracked file. Commit the print, measure writer-less numeric
   evidence such as product-surface boundary commit counts, update its values
   and number wording, and amend the same commit. Run `release/release/verify`
   after the amend. `rust/release-print-sensitive-evidence.json` is the complete
   registry of evidence that must travel with the print.
3. **Tag** — create `release-v<version>` only at the merged, verified commit.
   Pushing the immutable source selector starts no publication job.
4. **Dry-run first** — dispatch every publish workflow explicitly with its
   non-uploading path. The CLI default is stage-only; the crate train also runs
   the steady-state semver gate against live registry state.
5. **Publish with scoped provenance** — npm and extension workflows must be
   dispatched on the tag ref because their guards bind `github.sha` to checkout.
   The crate train and CLI may be dispatched from the default-branch workflow
   with `inputs.tag`: their tooling comes from the branch while source builds
   and the train's two guards resolve the immutable tag. Partial train failures
   resume with `resume`; registry versions are never overwritten.
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

The npm and extension dispatch ref and publication checkout must be the same
commit. Both publish workflows carry an inline guard immediately after checkout
that hard-fails before any publish step when `github.sha` differs from the
checked-out `HEAD` — dispatching a tag build from a moving branch silently
poisons the signed SLSA/attestation source metadata (learned irreversibly in
the 0.4.0 train; nine npm packages and the VSIX permanently attest a commit
that was not built). Dispatch those workflows from the tag ref itself. The
train is the explicit exception: default-branch tooling accepts `inputs.tag`,
then both jobs prove checked-out SHA equals tag SHA. CLI builds use the tag
while stage/upload use current tooling and an exported body.

## Release notes pipeline

- The source of truth is the authored release page in `docs/releases/` (the
  docs-site frontmatter is stripped at render time and never appears in a
  release body).
- The rendered body is: authored highlights → `## Changes` (generated:
  declared breaking changes from the semver-intent contract, then
  Features/Fixes/Performance from conventional-commit subjects over the
  compare range) → `## Distribution` (channel-specific facts from the
  manifest) → `## Release links` (source, changelog, full diff).
- The crate train and extension render with `--changelog`; CLI rebuilds export
  the already-persisted GitHub Release body and fail if it is absent. Every
  upload uses `body_path`, then `verify-github` checks byte equality.
- GitHub's `generate_release_notes` is never used for registered releases.

## Historical releases

The 60 legacy `v*` releases are frozen history: bodies carry a pre-rebrand
banner (and a pointer when a duplicate exists under the current families),
assets are never modified, and tags are never deleted. The
`release:notes backfill` command is the only sanctioned editor for published
bodies; it is idempotent and non-destructive.
