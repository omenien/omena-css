import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/publish-flags
 *
 * The crates.io face is the GENERATED standalone workspace, whose
 * `[workspace.package] publish` is flipped to true. In the SOURCE monorepo the
 * workspace is `publish = false`, so every member resolves to a not-publishable
 * `publish = []`. This gate enforces two invariants that keep the publish
 * surface honest:
 *
 *   (1) No source member escapes the workspace publish=false (no explicit
 *       `publish = true` / registry allow-list) — so the ONLY publish path is
 *       the generated train. Transfer-safe.
 *   (2) No [I] internal crate is a publish-train member — an [I] crate must
 *       never be copied into the generated workspace where it would inherit
 *       publish=true. (Jointly with rust/publish-train-closure.)
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const generatorSource = readFileSync(
  path.join(repoRoot, "scripts/prepare-omena-css-workspace.mjs"),
  "utf8",
);

function parseTrain(): Set<string> {
  const match = generatorSource.match(/const omenaCssCrates = \[([\s\S]*?)\];/);
  assert.ok(match, "expected omenaCssCrates in prepare-omena-css-workspace.mjs");
  return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]));
}

interface CargoPackage {
  readonly name: string;
  readonly publish: readonly string[] | null;
  readonly metadata?: { readonly omena?: { readonly role?: string } };
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const train = parseTrain();

// (1) Source publish hygiene: every member resolves to publish=[] (not
//     publishable). A null publish means "publishable to any registry" — that
//     would escape the workspace publish=false and is a hard error.
const escaping: string[] = [];
for (const pkg of metadata.packages) {
  const notPublishable = Array.isArray(pkg.publish) && pkg.publish.length === 0;
  if (!notPublishable) {
    escaping.push(`${pkg.name} resolves publish=${JSON.stringify(pkg.publish)} (expected [] via workspace publish=false)`);
  }
}
assert.equal(
  escaping.length,
  0,
  `source members must inherit the workspace publish=false (resolve to publish=[]); these escape it:\n  ${escaping.join("\n  ")}`,
);

// (2) No [I] internal crate is a publish-train member.
const internalInTrain: string[] = [];
for (const pkg of metadata.packages) {
  if (pkg.metadata?.omena?.role === "I" && train.has(pkg.name)) {
    internalInTrain.push(pkg.name);
  }
}
assert.equal(
  internalInTrain.length,
  0,
  `internal [I] crates must not be publish-train members (they would be flipped to publish=true in the generated workspace):\n  ${internalInTrain.join("\n  ")}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.publish-flags",
      members: metadata.packages.length,
      trainMembers: train.size,
      sourcePublishEscapes: 0,
      internalInTrain: 0,
    },
    null,
    2,
  )}\n`,
);
