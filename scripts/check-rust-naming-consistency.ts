import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/naming-consistency
 *
 * Brand discipline for the workspace member universe, derived purely from
 * `cargo metadata` + the role manifest (this gate no longer reads the retired
 * `prepare-omena-css-workspace.mjs` generator). Two invariants:
 *
 *   (1) Universe partition: every in-tree package name is either `omena-*` or one
 *       of a FROZEN, shrink-only `engine-*` allow-list (the remaining [I]
 *       non-published holdouts). The allow-list may only SHRINK as crates migrate
 *       to `omena-*`; any OTHER non-omena name is a hard failure with no escape
 *       hatch (prevents reintroducing a non-omena name for a new crate).
 *
 *   (2) Brand seam: every PUBLISHABLE member (publish != []) MUST be `omena-*`.
 *       Under Model A the crates.io face is the workspace itself, so a publishable
 *       crate publishes under its in-tree name — a non-omena publishable name would
 *       ship an off-brand crate to the registry. After step17 every publishable
 *       member is already `omena-*`; the two `engine-*` holdouts are non-publishable
 *       [I] and stay off crates.io.
 *
 * Stale allow-list entries (no longer present in the workspace) are reported, not
 * failed: a migrated-away `engine-*` crate is fine, the gate just surfaces dead
 * entries so they can be pruned.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// FROZEN, shrink-only allow-list of non-omena package names. Do NOT add to this.
const ENGINE_ALLOWLIST = new Set(["engine-shadow-runner", "engine-style-parser"]);

interface CargoPackage {
  readonly name: string;
  readonly publish: readonly string[] | null;
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

/** A member is publishable to crates.io iff its `publish` is not the empty array. */
function isPublishable(pkg: { readonly publish: readonly string[] | null }): boolean {
  return !(Array.isArray(pkg.publish) && pkg.publish.length === 0);
}

const memberNames = metadata.packages.map((pkg) => pkg.name);

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

// (2) Brand seam: every publishable member must be omena-* (the in-tree name IS the
//     published name under Model A direct publish).
const offBrandPublishable: string[] = [];
let publishable = 0;
for (const pkg of metadata.packages) {
  if (!isPublishable(pkg)) {
    continue;
  }
  publishable += 1;
  if (!pkg.name.startsWith("omena-")) {
    offBrandPublishable.push(
      `${pkg.name} (publishable but not omena-*; would ship off-brand to crates.io)`,
    );
  }
}
assert.equal(
  offBrandPublishable.length,
  0,
  `every publishable member must be branded omena-*:\n  ${offBrandPublishable.join("\n  ")}\n` +
    `Either rename the crate to omena-* or make it publish=false (an [I] holdout in the engine-* allow-list).`,
);

// (3) Shrink-only: an allow-list entry that no longer exists in the workspace is
//     fine (migrated away); the gate just must not be carrying dead entries that
//     could mask a regression. Report (do not fail) any stale allow-list entry.
const present = new Set(memberNames);
const staleAllowlist = [...ENGINE_ALLOWLIST].filter((name) => !present.has(name));

// Self-test: the publishable predicate, the partition, and the brand-seam checks.
{
  assert.equal(
    isPublishable({ publish: null }),
    true,
    "self-test: publish=null (default) is publishable",
  );
  assert.equal(
    isPublishable({ publish: ["crates-io"] }),
    true,
    "self-test: an explicit non-empty publish registry list is publishable",
  );
  assert.equal(
    isPublishable({ publish: [] }),
    false,
    "self-test: publish=[] (empty array) is NOT publishable",
  );

  // Partition: omena-* and allow-list members pass; any other non-omena name strays.
  const partitionStray = (name: string): boolean =>
    !name.startsWith("omena-") && !ENGINE_ALLOWLIST.has(name);
  assert.equal(partitionStray("omena-parser"), false, "self-test: omena-* member is not a stray");
  assert.equal(
    partitionStray("engine-shadow-runner"),
    false,
    "self-test: a frozen allow-list member is not a stray",
  );
  assert.equal(
    partitionStray("engine-input-producers"),
    true,
    "self-test: a non-omena name outside the allow-list is a stray",
  );

  // Brand seam: a publishable non-omena member is off-brand; a non-publishable one is not.
  const isOffBrand = (pkg: { name: string; publish: readonly string[] | null }): boolean =>
    isPublishable(pkg) && !pkg.name.startsWith("omena-");
  assert.equal(
    isOffBrand({ name: "engine-shadow-runner", publish: [] }),
    false,
    "self-test: a non-publishable [I] holdout is not off-brand",
  );
  assert.equal(
    isOffBrand({ name: "engine-shadow-runner", publish: null }),
    true,
    "self-test: a publishable non-omena member IS off-brand",
  );
  assert.equal(
    isOffBrand({ name: "omena-cli", publish: null }),
    false,
    "self-test: a publishable omena-* member is on-brand",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.naming-consistency",
      members: memberNames.length,
      publishable,
      nonOmenaNames: memberNames.filter((name) => !name.startsWith("omena-")),
      strayNames: 0,
      offBrandPublishable: 0,
      staleAllowlistEntries: staleAllowlist,
    },
    null,
    2,
  )}\n`,
);
