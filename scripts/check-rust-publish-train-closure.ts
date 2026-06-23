import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/publish-train-closure
 *
 * Model A (direct publish): the monorepo IS the crates.io publish source, so the
 * publish train is simply the PUBLISHABLE set of workspace members — every member
 * whose resolved `publish` is not `[]` (publish-flags pins that set from Cargo
 * metadata). There is no longer a generated standalone
 * workspace, and no hand-maintained `omenaCssCrates` / `omenaCssPublishOrder`
 * literal to keep in lock-step. This gate derives everything from `cargo metadata`.
 *
 * It proves two structural facts plus a drift guard:
 *
 *   - (closure) For every publishable crate, every NON-dev path-dep target that is a
 *     workspace member must ITSELF be publishable. Otherwise that publishable crate
 *     could not `cargo publish`: its dependency would not exist on crates.io. (A
 *     dev-dep is exempt — cargo strips its path at publish and requires no version.)
 *   - (canonical order) The deterministic Kahn + lexicographic-tie-break topological
 *     order over the train-internal non-dev path-dep edges (deps first) — the genesis
 *     wave publish sequence. It is EMITTED in the JSON output (a machine derivation,
 *     no longer a literal to compare against).
 *   - (edge-set sha) A sha256 of the sorted train shipped-path-dep edge set, pinned to
 *     EXPECTED_EDGE_SET_SHA256. Any added/removed shipped path-dep edge changes the
 *     hash, forcing a DELIBERATE bump — graph drift, distinct from a mere set change.
 *
 * It walks the WIDE dependency set (normal + build + optional path-deps; dev-deps do
 * not ship), NOT the feature-resolved graph, so an optional path-dep to a
 * non-publishable crate is still caught. A self-test guards the closure predicate.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
  readonly path?: string;
  readonly optional: boolean;
}
interface CargoPackage {
  readonly name: string;
  readonly publish: readonly string[] | null;
  readonly dependencies: readonly CargoDependency[];
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const packagesByName = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));
const memberNames = new Set(metadata.packages.map((pkg) => pkg.name));

/** A member is publishable iff its resolved `publish` is NOT the empty array. */
function isPublishable(pkg: { readonly publish: readonly string[] | null }): boolean {
  return !(Array.isArray(pkg.publish) && pkg.publish.length === 0);
}

// The publish train: the publishable set of workspace members (sorted for a stable
// canonical-order seed; the Kahn tie-break re-sorts anyway).
const trainCrates = metadata.packages
  .filter(isPublishable)
  .map((pkg) => pkg.name)
  .toSorted();
const trainSet = new Set(trainCrates);

/** Non-dev path-dep targets of a crate that are themselves workspace members (the set that must ship together). */
function shippedMemberPathDeps(crate: string): string[] {
  const pkg = packagesByName.get(crate);
  assert.ok(pkg, `train crate ${crate} not found in the rust/ workspace cargo metadata`);
  return pkg.dependencies
    .filter(
      (dep) => dep.kind !== "dev" && typeof dep.path === "string" && memberNames.has(dep.name),
    )
    .map((dep) => dep.name);
}

/**
 * Shared closure predicate (also exercised by the self-test below): a train edge is a
 * closure GAP when its non-dev path-dep target is a workspace member that is NOT itself
 * in the train — that target is not on crates.io, so the train crate could not publish.
 */
function closureGaps(
  train: ReadonlySet<string>,
  members: ReadonlySet<string>,
  edges: ReadonlyArray<readonly [string, string]>,
): string[] {
  return edges
    .filter(([from, to]) => train.has(from) && members.has(to) && !train.has(to))
    .map(([from, to]) => `${from} -> ${to}`);
}

/**
 * Deterministic Kahn + lexicographic-tie-break topological order over the train-internal
 * shipped-path-dep edges: each crate appears AFTER all of its train-internal deps, with
 * ties broken lexicographically. This is the canonical publish sequence — independent of
 * cargo-metadata iteration order — i.e. the genesis wave order.
 */
function computeCanonicalOrder(
  members: readonly string[],
  train: ReadonlySet<string>,
  allEdges: ReadonlyArray<readonly [string, string]>,
): string[] {
  const remaining = new Map(members.map((crate) => [crate, new Set<string>()]));
  const dependents = new Map<string, string[]>(members.map((crate) => [crate, []]));
  for (const [from, to] of allEdges) {
    if (train.has(to) && from !== to) {
      remaining.get(from)!.add(to);
      dependents.get(to)!.push(from);
    }
  }
  const ready = members.filter((crate) => remaining.get(crate)!.size === 0);
  const order: string[] = [];
  while (ready.length > 0) {
    ready.sort();
    const next = ready.shift()!;
    order.push(next);
    for (const dependent of dependents.get(next)!) {
      const rest = remaining.get(dependent)!;
      rest.delete(next);
      if (rest.size === 0) {
        ready.push(dependent);
      }
    }
  }
  return order;
}

// Build the real edge set: every non-dev path-dep of every train crate whose target is a
// workspace member (shipped path-deps; only member targets can break the train).
const edges: Array<readonly [string, string]> = [];
for (const crate of trainCrates) {
  for (const target of shippedMemberPathDeps(crate)) {
    edges.push([crate, target]);
  }
}

// (1) Closure: every shipped path-dep target of a train crate is itself publishable.
const closureViolations = closureGaps(trainSet, memberNames, edges);
assert.equal(
  closureViolations.length,
  0,
  `publish-train closure gap: ${closureViolations.length} shipped path-dep(s) of publishable crates target a member that is NOT publishable — \`cargo publish\` would fail because that dependency is not on crates.io:\n  ${closureViolations.join(
    "\n  ",
  )}\nMake the target publishable (publish flag / role) or remove the non-dev path-dep.`,
);

// (2) Canonical order: the deterministic Kahn + lexicographic-tie-break order (deps first).
//     This is the genesis wave publish sequence; it is emitted below, not compared to a literal.
const canonicalOrder = computeCanonicalOrder(trainCrates, trainSet, edges);
assert.equal(
  canonicalOrder.length,
  trainCrates.length,
  `canonical publish order length ${canonicalOrder.length} != train size ${trainCrates.length} — ` +
    "the train-internal shipped-path-dep graph has a cycle (no valid publish order).",
);

// (3) Edge-set hash: pin the train-graph shape. Any added/removed shipped path-dep edge
//     changes this hash, forcing a DELIBERATE bump — graph drift, distinct from set drift.
const edgeSetSha256 = createHash("sha256")
  .update(
    edges
      .map(([from, to]) => `${from}->${to}`)
      .toSorted()
      .join("\n"),
  )
  .digest("hex");
const EXPECTED_EDGE_SET_SHA256 = "8e16a04e671fd0108b377ab53a245aa79916014508d8fe6688b1193c3f2ff170";
assert.equal(
  edgeSetSha256,
  EXPECTED_EDGE_SET_SHA256,
  "train shipped-path-dep edge set changed (graph drift).\n" +
    "If intended, update EXPECTED_EDGE_SET_SHA256 to: " +
    edgeSetSha256,
);

// (4) Self-test: the closure predicate must flag a path-dep to a non-train MEMBER, must
//     NOT flag a target that is publishable (in-train), and must NOT flag a non-member target.
{
  const probeTrain = new Set(["probe-train-crate"]);
  const probeMembers = new Set(["probe-train-crate", "probe-nonpub-member"]);
  const gapEdge: ReadonlyArray<readonly [string, string]> = [
    ["probe-train-crate", "probe-nonpub-member"],
  ];
  assert.equal(
    closureGaps(probeTrain, probeMembers, gapEdge).length,
    1,
    "self-test failed: closure predicate did not flag a path-dep to a non-publishable member",
  );
  assert.equal(
    closureGaps(probeMembers, probeMembers, gapEdge).length,
    0,
    "self-test failed: closure predicate flagged an edge to a publishable (in-train) target",
  );
  assert.equal(
    closureGaps(probeTrain, probeTrain, [["probe-train-crate", "probe-train-crate"]]).length,
    0,
    "self-test failed: closure predicate flagged an in-train self edge",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.publish-train-closure",
      members: metadata.packages.length,
      trainCrateCount: trainCrates.length,
      shippedPathDepEdges: edges.length,
      closureViolations: 0,
      edgeSetSha256,
      canonicalPublishOrder: canonicalOrder,
    },
    null,
    2,
  )}\n`,
);
