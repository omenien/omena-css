---
title: Release notes
description: Curated Omena release notes and the manifest-backed publishing contract.
kind: how-to
status: stable
products: [release]
owner: release
sourceOfTruth: hybrid
---

# Release notes

`manifest.json` is the release-note authority for every new GitHub Release.
Publishing from an agent, a local operator, or GitHub Actions follows the same
contract: an exact tag must be registered, its notes file must be reviewed, and
the rendered body must pass the repository gate before any registry upload or
GitHub Release mutation.

## Authoring a release

1. Add or update the user-facing release document in this directory.
2. Register each independently published tag in `manifest.json`. The editor uses
   `vscode-vX.Y.Z`; the Rust crate train and CLI use `release-vX.Y.Z`.
3. Render the body locally:

   ```sh
   pnpm release:notes -- --tag vscode-v5.3.0 --output /tmp/release-notes.md
   ```

4. Run the release-note and crate-documentation gates:

   ```sh
   pnpm omena-check run release/check/release-notes
   pnpm omena-check run rust/crate-documentation
   ```

The renderer adds channel-specific artifacts and immutable source links to the
curated document. Preview extension tags reuse the registered stable document
and receive an explicit prerelease warning.

## Required content

A release document must explain:

- user-visible additions and corrections;
- defaults versus explicit opt-ins;
- migrations or compatibility boundaries;
- the versions and channels included in the release.

Do not use generated commit subjects as the sole release body. GitHub-generated
notes remain a useful fallback for historical backfills, but current releases
must have a registered, reviewed document.

## Repairing historical releases

The backfill command is dry-run by default. It preserves substantive existing
bodies, fills empty releases from the matching changelog section or GitHub's
generated comparison, and adds normalized source/changelog links.

```sh
node --import tsx ./scripts/release-notes.ts backfill
node --import tsx ./scripts/release-notes.ts backfill --tag v4.1.24 --print
node --import tsx ./scripts/release-notes.ts backfill --apply
```

The command is idempotent. Run it with an authenticated `gh` session and review
the dry-run summary before applying remote changes.

A manual `Release CLI` rebuild checks out the requested immutable tag only for
the binaries. Its release job uses the current tooling and preserves the
already-curated GitHub Release body, so rebuilding a historical artifact cannot
replace its notes or require old commits to contain the current manifest.
