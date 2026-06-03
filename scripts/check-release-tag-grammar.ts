import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * release/tag-grammar
 *
 * Prefixed release-tag grammar so the two independent release axes can never
 * collide on a shared `v*` namespace:
 *
 *   - `release-v<x.y.z>` → the crate-train + omena-cli release (release-cli.yml
 *     `on.push.tags`). Axis: crate train 0.x.
 *   - `vscode-v<x.y.z>`  → the VS Code extension VSIX release (publish-extension.yml
 *     `tag_name`). Axis: VSIX 5.x.
 *
 * A bare `v*` tag would be caught by BOTH (release-cli.yml historically fired on
 * `v*`, and publish-extension.yml created `v${VERSION}` tags), so publishing the
 * VSIX would have ACCIDENTALLY triggered the CLI/train release. This gate pins both
 * prefixes and forbids reintroducing the bare-`v*` form, in either workflow.
 *
 * The workflows are parsed textually (the repo has no node YAML dependency; this
 * mirrors the string-based workflow assertions in check-rust-m6-publication-material.ts
 * and check-rust-closure-fast-aggregation-complete.ts).
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (rel: string): string => readFileSync(path.join(repoRoot, rel), "utf8");

const releaseCli = read(".github/workflows/release-cli.yml");
const publishExt = read(".github/workflows/publish-extension.yml");

// (1) release-cli.yml fires on `release-v*`, never the bare `v*`.
assert.match(
  releaseCli,
  /tags:\s*\n\s*-\s*"release-v\*"/,
  'release-cli.yml on.push.tags must be "release-v*" (the crate-train + cli axis)',
);
assert.ok(
  !/tags:\s*\n\s*-\s*"v\*"/.test(releaseCli),
  'release-cli.yml must NOT fire on the bare "v*" tag — it would also catch vscode-v* / any v-prefixed tag and run the CLI release by mistake',
);

// (2) publish-extension.yml tags the VSIX with `vscode-v*`, never the bare `v${VERSION}`.
assert.ok(
  /TAG="vscode-v\$\{VERSION\}"/.test(publishExt),
  "publish-extension.yml stable TAG must be vscode-v${VERSION}",
);
assert.ok(
  /TAG="vscode-v\$\{VERSION\}-preview/.test(publishExt),
  "publish-extension.yml preview TAG must be vscode-v${VERSION}-preview.${GITHUB_RUN_NUMBER}",
);
assert.ok(
  !/TAG="v\$\{VERSION\}"/.test(publishExt),
  "publish-extension.yml must NOT use a bare v${VERSION} tag — it would trigger release-cli.yml",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.tag-grammar",
      crateTrainTagPrefix: "release-v",
      vsixTagPrefix: "vscode-v",
      bareVTagForbidden: true,
    },
    null,
    2,
  )}\n`,
);
