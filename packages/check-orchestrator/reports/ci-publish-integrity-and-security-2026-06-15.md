# CI Publish Integrity And Workflow Security

Generated during the 2026-06-15 CI architecture hardening pass.

## Release-Integrity Verdict

The M5 release gates and `cargo-semver-checks` are complementary, not redundant.

The M5 gates validate release-claim and product-surface invariants:

- `release/check/release-m5-class-value-universe-matrix`
- `release/check/release-m5-api-freeze-audit`
- `core/check`
- `test/test`

`cargo-semver-checks` is a Rust crate API compatibility check. It should be
added as a separate version-aware steady-state crate-train gate, but it does
not replace the M5 release gates because it does not validate npm package
metadata, VSIX packaging, class-value universe release claims, or the
cross-channel product release boundary.

## Channel Status

| Channel                        | Release-integrity status                                                                                                                  |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| VS Code Marketplace / Open VSX | Enforced by the extension publish path before `vsce publish` / `ovsx publish`.                                                            |
| crates.io crate train          | Enforced by `_publish-crate-train.yml` before the protected `publish` job reaches `cargo publish --workspace --locked`.                   |
| npm packages                   | Enforced by `_publish-npm.yml` before `@omena/wasm`, `@omena/napi`, native optional packages, or build-tool packages reach `npm publish`. |

## Failure Mode

The release-integrity jobs run before irreversible registry uploads. If a gate
fails after a `release-v*` tag has already been pushed, the maintainer should
fix the source issue, delete and re-cut the tag when no registry upload happened,
and re-run the publish workflow from the corrected tag.

If a crate-train recovery run has already partially published crates, keep using
the existing resume exclude path instead of adding another resume mechanism:
`_publish-crate-train.yml` computes the exclude set and passes the resulting
`--exclude` arguments to the workspace publish action.

npm package versions are immutable. If an npm upload already happened before a
failure is discovered, publish a corrected later version rather than attempting
to overwrite the existing one.

## Orchestrator Boundary

New publish-integrity gates use canonical `pnpm omena-check run <id>` targets.
The legacy extension shell script still calls compatibility script aliases
directly; shell-script bypass linting is a follow-up so this pass does not
expand the live publish-path blast radius.

## Workflow-Security Disposition

The first workflow-security lane is advisory:

- `workflow-security.yml` runs `zizmorcore/zizmor-action` with
  `advanced-security: false`.
- The zizmor step uses `continue-on-error: true`.
- The workflow uses read-only repository permissions and no release secrets.

Initial publish-machinery dispositions:

- Local probe: `uvx zizmor --format json --min-severity high --no-exit-codes .github/workflows .github/actions` returned 0 HIGH findings on this branch.
- `id-token: write` is accepted for OIDC/provenance publish paths.
- `contents: write` is accepted where the workflow creates GitHub releases.
- Long-lived release secrets and repository-admin/OIDC configuration remain
  user-gated and are not changed by this pass.
- Trusted-input interpolation hardening remains advisory for future findings.
