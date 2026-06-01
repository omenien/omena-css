import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/naming-consistency
 *
 * Brand discipline for the workspace member universe. Every in-tree package name
 * is either `omena-*` or one of a FROZEN, shrink-only `engine-*` allow-list (the
 * remaining [I] non-published holdouts). The allow-list may only SHRINK as crates
 * migrate to `omena-*`; any OTHER non-omena name is a hard failure with no escape
 * hatch (prevents reintroducing a non-omena name for a new crate).
 *
 * Every PUBLISHED (train) crate resolves to an `omena-*` published name directly:
 * all train members are now `omena-*` in-tree (engine-input-producers was renamed
 * to omena-engine-input-producers), so PUBLISH_RENAME is empty. The two remaining
 * `engine-*` holdouts are [I] and stay out of the train.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const generatorSource = readFileSync(
  path.join(repoRoot, "scripts/prepare-omena-css-workspace.mjs"),
  "utf8",
);

// FROZEN, shrink-only allow-list of non-omena package names. Do NOT add to this.
const ENGINE_ALLOWLIST = new Set(["engine-shadow-runner", "engine-style-parser"]);

// Known publish-time renames for non-omena train members (the brand seam): the
// in-tree package name -> its omena-* published name. Now EMPTY — every train
// member is `omena-*` in-tree (engine-input-producers -> omena-engine-input-producers
// landed in-tree), and the two remaining engine-* holdouts are [I], not train members.
const PUBLISH_RENAME = new Map<string, string>([]);

function parseTrain(): Set<string> {
  const match = generatorSource.match(/const omenaCssCrates = \[([\s\S]*?)\];/);
  assert.ok(match, "expected omenaCssCrates in prepare-omena-css-workspace.mjs");
  return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]));
}

interface CargoPackage {
  readonly name: string;
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const memberNames = metadata.packages.map((pkg) => pkg.name);
const train = parseTrain();

// (1) Universe partition: every member is omena-* or in the frozen allow-list.
const strayNames: string[] = [];
for (const name of memberNames) {
  if (!name.startsWith("omena-") && !ENGINE_ALLOWLIST.has(name)) {
    strayNames.push(name);
  }
}
assert.equal(
  strayNames.length,
  0,
  `non-omena package names must be in the frozen engine-* allow-list; these are not:\n  ${strayNames.join("\n  ")}`,
);

// (2) Shrink-only: an allow-list entry that no longer exists in the workspace is
//     fine (migrated away); the gate just must not be carrying dead entries that
//     could mask a regression. Report (do not fail) any stale allow-list entry.
const present = new Set(memberNames);
const staleAllowlist = [...ENGINE_ALLOWLIST].filter((name) => !present.has(name));

// (3) Every train member resolves to an omena-* published name.
const badPublishedNames: string[] = [];
for (const crate of train) {
  const publishedName = crate.startsWith("omena-") ? crate : PUBLISH_RENAME.get(crate);
  if (publishedName === undefined || !publishedName.startsWith("omena-")) {
    badPublishedNames.push(`${crate} (no omena-* published name; not omena-* and no rename entry)`);
  }
}
assert.equal(
  badPublishedNames.length,
  0,
  `every publish-train member must publish under an omena-* name:\n  ${badPublishedNames.join("\n  ")}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.naming-consistency",
      members: memberNames.length,
      nonOmenaNames: memberNames.filter((name) => !name.startsWith("omena-")),
      strayNames: 0,
      staleAllowlistEntries: staleAllowlist,
      trainMembers: train.size,
      trainPublishedNameViolations: 0,
    },
    null,
    2,
  )}\n`,
);
