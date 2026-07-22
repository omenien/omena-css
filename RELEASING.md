# Releasing

This is the maintainer runbook for publishing the Rust crate train, npm
packages, VS Code extension, and Open VSX extension. Run releases from a clean,
reviewed commit; registry uploads are not a substitute for CI evidence.

## Version Axes

The project has three independent version lines:

| Axis                       | Source of truth                     | Tag              | Published artifacts                            |
| -------------------------- | ----------------------------------- | ---------------- | ---------------------------------------------- |
| Rust crate train           | `rust/Cargo.toml` workspace version | `release-vX.Y.Z` | crates.io, CLI archives, npm bindings/plugins  |
| VS Code extension          | root `package.json`                 | `vscode-vX.Y.Z`  | Marketplace, Open VSX, VSIX, SBOM, attestation |
| Private TypeScript tooling | package-local manifests             | none             | not published                                  |

The machine-checked axis and reservation policy is documented in
[`docs/governance/version-governance.md`](docs/governance/version-governance.md).
The registry crate baseline is `0.2.0` and the published npm binding baseline is
`0.2.1`; the source workspace may carry the next unpublished crate-train
version. `5.2.0` is the published extension baseline, with tag
`vscode-v5.2.0`. The coordinated source candidate is crate/npm `0.3.0` and
extension `5.3.0` until channel publication is accepted.

Earlier releases must not be reused as closure artifacts. The `5.0.0` and
`5.1.x` tags remain historical records.

Historical extension ledger: `3.2.0`, `3.2.1`, `3.3.0`, `3.4.0`, `3.5.0`,
`3.6.0`, `3.7.0`, `3.8.0`, `3.9.0`, `3.10.0`, `3.11.0`, `3.12.0`, `3.13.0`,
`3.14.0`, `3.15.0`, `4.0.0`, `4.1.0`, `5.0.0`, `5.1.0`, `5.1.1`. This ledger
is historical evidence, not a list of reusable versions.

## Before Publishing

1. Confirm the release commit is on `master` for stable or `next` for preview,
   and that the worktree and submodules are clean.
2. Put user-visible changes under the matching version in `CHANGELOG.md` and
   review any changeset-generated version commit.
3. Confirm `package.json`, `rust/Cargo.toml`, exact inter-crate pins, lockfiles,
   generated package manifests, and intended tags agree with the selected
   release axis. Run `pnpm omena-check run docs/version-governance` to check the
   documented policy against those authorities.
4. Install the committed dependency graph and run the release bundle:

```bash
pnpm install --frozen-lockfile
pnpm release:verify
```

5. Review the produced VSIX contents, native target matrix, package tarballs,
   provenance subjects, and the green CI run for the exact commit.
6. Perform each registry's dry-run before enabling an irreversible publish.

`pnpm release:verify` synchronizes server metadata, enforces release wording and
class-value evidence, builds product artifacts, runs core/plugin/Rust/tsgo/test
gates, packages the VSIX, and verifies the packaged Rust LSP/type-fact path.

## Four-Channel Checklist

### crates.io

- Bump the workspace version and exact inter-crate pins together. Run the
  `rust/publish-train-closure` and `rust/inter-crate-pin` gates through release
  verification.
- Dispatch `_Publish Crate Train` with `mode=oidc`, `dry_run=true`, and
  `resume=false`. Review the canonical publish order and every package dry-run.
- Push `release-vX.Y.Z` only after the dry-run is green. The tag starts the crate
  publish and the five-target `Release CLI` archive/checksum workflow.
- Existing crate names use crates.io Trusted Publishing. A never-published name
  requires one `mode=bootstrap` run with the protected `CRATES_IO_TOKEN`, then
  registration for OIDC.
- Confirm every publishable crate at the exact version, the sparse-index poll,
  install smoke, GitHub release, CLI archives, and checksums.
- Publishing is non-atomic and irreversible. If a train stops after partial
  upload, re-dispatch with `resume=true`; do not reuse the version.

### npm

- Dispatch `_Publish npm` for the exact release ref with `dry_run=true`.
  Select `publish_wasm`, `publish_napi`, and `publish_plugins` deliberately.
- Inspect packed names, versions, repository URLs, native optional-dependency
  names, and the five NAPI target artifacts before setting `dry_run=false`.
- `@omena/wasm` and `@omena/napi` use Trusted Publishing where configured.
  First-publish platform packages and build-tool packages use the protected
  `NPM_AUTO_TOKEN`; all uploads include npm provenance.
- Confirm `@omena/napi` declares every published platform package. An immutable
  main package with an incomplete optional-dependency map requires a new
  crate-train version.
- Re-dispatching is safe only because the workflow checks `npm view` and skips
  already-published package/version pairs. Never overwrite a registry version.

### VS Code Marketplace

- Bump the root extension version and update `CHANGELOG.md`. Keep the crate axis
  unchanged unless Rust/npm artifacts are also being released.
- Run the `Publish Extension` workflow with the exact ref, `channel=stable`,
  `publish_marketplace=true`, and the desired GitHub-release setting.
- Confirm the merged Linux/macOS/Windows Rust runner, LSP server, and tsgo matrix
  is inside the staged VSIX before upload.
- Confirm Marketplace version `omena.omena-css`, tag `vscode-vX.Y.Z`, VSIX,
  CycloneDX SBOM, and build-provenance attestation all reference the same commit.

### Open VSX

- Use the same `Publish Extension` workflow, exact ref, and already-validated
  VSIX. Set `publish_openvsx=true`; do not rebuild a different artifact.
- Confirm `OVSX_PAT` is available only to the publish step and the resulting
  `omena.omena-css` version matches Marketplace when both channels are enabled.
- Open VSX preview behavior is not treated as equivalent to Marketplace preview.
  Verify preview visibility manually before announcing that channel.

## Preview Releases

Preview releases use a unique numeric `major.minor.patch` version on `next` and
`channel=preview`; the workflow adds `--pre-release` and a preview GitHub tag.
A preview version must never be reused for stable. Marketplace is the primary
preview channel; enabling Open VSX preview is an explicit operator decision.

## Release claim discipline

Public release text describes shipped behavior and evidence, not internal
substrates. Avoid internal milestone labels, planning shorthand, and P-numbering
in README, CHANGELOG, release notes, and registry descriptions.

- Map every user-visible claim to a pushed commit, green gate, package check, or
  published artifact.
- Treat V0 contracts as internal unless a product path exercises them through a
  shipped extension, CLI, crate, or SDK surface.
- For issue #61, release text may mention only the Finding-D class-value-universe
  substrate when its evidence matrix is green. Do not close or describe the broader #61 resolver/Sass/
  workspace/paradigm RFC as complete without separate product evidence.
- Automation and testkit surfaces are release-framed only when their fixture
  grammar, schema, known-failure policy, and failure modes are gated.
- Cargo crate versioning stays on the gradual `0.x` line. Breaking public
  contracts advance the minor version; compatible fixes may advance the patch.
  Do not publish or describe a Cargo `1.0.0` API-freeze line until a train-wide
  public API freeze artifact and review exist.

`pnpm check:release-m5-class-value-universe-matrix` is the release-facing
fixture matrix for the issue #61 Finding-D slice. It verifies CSS Modules finite
fallback, vanilla-extract recipes, and cva phase 1 class-value universes while
recording the slots axis as reserved/deferred.

`pnpm check:release-m5-api-freeze-audit` is the release/API-freeze wording
gate. Both checks are included in `pnpm release:verify` and in publish integrity
jobs.

Do not claim a public Datalog host, egglog binding, modal theorem prover,
belief-propagation result, safety margin, or final external plugin ABI from
research contracts alone.

## Failure Recovery

- Crate partial publish: use `resume=true` at the same immutable tag; never try
  to delete or replace published bytes.
- npm partial publish: re-dispatch the exact ref; existing versions are skipped.
- Extension registry failure: reuse the uploaded workflow artifact and exact
  commit. Do not package from a dirty checkout.
- GitHub release failure after registry success: rerun only the release creation
  path against the existing tag and artifacts.
- Any unexpected digest, package list, optional dependency, or provenance
  subject is a stop condition, not a warning to waive.

## Local Operator Path

The local wrapper is available for reproducing extension packaging and publish
selection, but hosted publication remains the normal path:

```bash
RELEASE_CHANNEL=stable \
PUBLISH_MARKETPLACE=false \
PUBLISH_OPENVSX=false \
pnpm release:publish
```

Marketplace and Open VSX publication require `VSCE_PAT` and `OVSX_PAT`. The
wrapper reads a repo-root `.env` when present; never commit credentials.

## Changesets And Maintenance

User-facing changes should include a changeset. Documentation-, test-, CI-, and
example-only pull requests may use `changeset:skip`. The Release Plan workflow
creates the version commit; review it rather than editing generated versions in
parallel.

Contributor extension recipes and focused validation commands live in
[CONTRIBUTING.md](CONTRIBUTING.md). The full generated check inventory is
[packages/check-orchestrator/CHECKS.md](packages/check-orchestrator/CHECKS.md).
