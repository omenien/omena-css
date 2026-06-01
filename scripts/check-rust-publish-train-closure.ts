import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/publish-train-closure
 *
 * The omena-css publish train is the GENERATED standalone workspace produced by
 * scripts/prepare-omena-css-workspace.mjs. Its membership lives in two
 * hand-maintained literals there: `omenaCssCrates` (copy set) and
 * `omenaCssPublishOrder` (publish sequence). When a train crate path-depends on
 * a workspace crate that is NOT itself a train member (nor externally
 * published), the generated workspace carries an UNVERSIONABLE path-dep and the
 * next publish breaks. This gate proves, mechanically, that the train is
 * dependency-closed and the two literals stay in lock-step + topological order.
 *
 * It walks the WIDE dependency set from `cargo metadata` (normal + build +
 * optional path-deps; dev-deps do not ship), NOT the feature-resolved graph, so
 * an optional path-dep to a non-train crate is still caught. A self-test guards
 * the detection predicate itself.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const generatorPath = path.join(repoRoot, "scripts/prepare-omena-css-workspace.mjs");
const generatorSource = readFileSync(generatorPath, "utf8");

function parseStringArray(name: string): string[] {
  const match = generatorSource.match(new RegExp(`const ${name} = \\[([\\s\\S]*?)\\];`));
  assert.ok(match, `expected a \`const ${name} = [...]\` literal in ${generatorPath}`);
  return [...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function parseStringSet(name: string): Set<string> {
  const match = generatorSource.match(
    new RegExp(`const ${name} = new Set\\(\\[([\\s\\S]*?)\\]\\)`),
  );
  assert.ok(match, `expected a \`const ${name} = new Set([...])\` literal in ${generatorPath}`);
  return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]));
}

const trainCrates = parseStringArray("omenaCssCrates");
const publishOrder = parseStringArray("omenaCssPublishOrder");
const externalCrates = parseStringSet("externallyPublishedCrates");
const trainSet = new Set(trainCrates);

assert.equal(trainSet.size, trainCrates.length, "omenaCssCrates contains duplicate entries");

// (1) Lock-step: the publish order is a permutation of the copy set (same members).
{
  const publishSet = new Set(publishOrder);
  assert.equal(
    publishSet.size,
    publishOrder.length,
    "omenaCssPublishOrder contains duplicate entries",
  );
  assert.equal(
    publishOrder.length,
    trainCrates.length,
    `omenaCssPublishOrder length ${publishOrder.length} != omenaCssCrates length ${trainCrates.length}`,
  );
  for (const crate of trainSet) {
    assert.ok(
      publishSet.has(crate),
      `train crate ${crate} is in omenaCssCrates but missing from omenaCssPublishOrder`,
    );
  }
  for (const crate of publishSet) {
    assert.ok(
      trainSet.has(crate),
      `${crate} is in omenaCssPublishOrder but missing from omenaCssCrates`,
    );
  }
}

interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
  readonly path?: string;
  readonly optional: boolean;
}
interface CargoPackage {
  readonly name: string;
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

/** Non-dev path-dep targets of a workspace crate (the set that must ship together). */
function shippedPathDeps(crate: string): string[] {
  const pkg = packagesByName.get(crate);
  assert.ok(pkg, `train crate ${crate} not found in the rust/ workspace cargo metadata`);
  return pkg.dependencies
    .filter((dep) => dep.kind !== "dev" && typeof dep.path === "string")
    .map((dep) => dep.name);
}

/** Shared detection predicate (also exercised by the self-test below). */
function closureGaps(
  train: ReadonlySet<string>,
  external: ReadonlySet<string>,
  edges: ReadonlyArray<readonly [string, string]>,
): string[] {
  return edges
    .filter(([from, to]) => train.has(from) && !train.has(to) && !external.has(to))
    .map(([from, to]) => `${from} -> ${to}`);
}

/**
 * Deterministic Kahn + lexicographic-tie-break topological order over the train-internal
 * shipped-path-dep edges: each crate appears AFTER all of its train-internal deps, with
 * ties broken lexicographically. This is the canonical publish sequence — independent of
 * cargo-metadata iteration order — that omenaCssPublishOrder must equal.
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

// Build the real edge set: every non-dev path-dep of every train crate.
const edges: Array<readonly [string, string]> = [];
for (const crate of trainCrates) {
  for (const target of shippedPathDeps(crate)) {
    edges.push([crate, target]);
  }
}

// (2) Closure: every shipped path-dep target of a train crate is itself a train
//     member or externally published.
const closureViolations = closureGaps(trainSet, externalCrates, edges);
assert.equal(
  closureViolations.length,
  0,
  `publish-train closure gap: ${closureViolations.length} shipped path-dep(s) of train crates are neither train members nor externally published:\n  ${closureViolations.join(
    "\n  ",
  )}\nAdd the missing crate(s) to omenaCssCrates + omenaCssPublishOrder (topologically) or to externallyPublishedCrates.`,
);

// (3) Topological order: in omenaCssPublishOrder, every train crate is published
//     AFTER all of its train-internal shipped path-deps.
const positionInOrder = new Map(publishOrder.map((crate, index) => [crate, index]));
const orderViolations: string[] = [];
for (const [from, to] of edges) {
  if (!trainSet.has(to)) {
    continue; // non-train targets are covered by the closure check above
  }
  const fromPosition = positionInOrder.get(from)!;
  const toPosition = positionInOrder.get(to)!;
  if (toPosition >= fromPosition) {
    orderViolations.push(
      `${from} (publish pos ${fromPosition}) must come AFTER its dep ${to} (publish pos ${toPosition})`,
    );
  }
}
assert.equal(
  orderViolations.length,
  0,
  `omenaCssPublishOrder is not topologically valid (a crate publishes before one of its deps):\n  ${orderViolations.join(
    "\n  ",
  )}`,
);

// (3b) Canonical order: omenaCssPublishOrder must EQUAL the deterministic Kahn +
//      lexicographic-tie-break order (deps first), not merely be A valid order. This pins
//      the literal to a machine derivation so ordering drift is caught, not just invalidity.
const canonicalOrder = computeCanonicalOrder(trainCrates, trainSet, edges);
const canonicalMatches =
  canonicalOrder.length === publishOrder.length &&
  canonicalOrder.every((crate, index) => crate === publishOrder[index]);
assert.ok(
  canonicalMatches,
  "omenaCssPublishOrder is topologically valid but is NOT the canonical Kahn+lexicographic order.\n" +
    "Replace the omenaCssPublishOrder literal in scripts/prepare-omena-css-workspace.mjs with:\n" +
    canonicalOrder.map((crate) => `  "${crate}",`).join("\n"),
);

// (3c) Edge-set hash: pin the train-graph shape. Any added/removed shipped path-dep edge
//      changes this hash, forcing a DELIBERATE bump — graph drift, distinct from order drift.
const edgeSetSha256 = createHash("sha256")
  .update(
    edges
      .map(([from, to]) => `${from}->${to}`)
      .toSorted()
      .join("\n"),
  )
  .digest("hex");
const EXPECTED_EDGE_SET_SHA256 = "76f438e0dd84707489ee099995757708b60ed28fa5a6a11923cb919d4f114f39";
assert.equal(
  edgeSetSha256,
  EXPECTED_EDGE_SET_SHA256,
  "train shipped-path-dep edge set changed (graph drift).\n" +
    "If intended, regenerate omenaCssPublishOrder (the canonical order is printed on an order mismatch)\n" +
    `and update EXPECTED_EDGE_SET_SHA256 to: ${edgeSetSha256}`,
);

// (4) Self-test: the detection predicate must flag a non-train path-dep, must
//     NOT flag an externally-published target, and must NOT flag an in-train edge.
{
  const probeTrain = new Set(["probe-train-crate"]);
  const gapEdge: ReadonlyArray<readonly [string, string]> = [
    ["probe-train-crate", "probe-nontrain-dep"],
  ];
  assert.equal(
    closureGaps(probeTrain, new Set(), gapEdge).length,
    1,
    "self-test failed: closure predicate did not flag a path-dep to a non-train crate",
  );
  assert.equal(
    closureGaps(probeTrain, new Set(["probe-nontrain-dep"]), gapEdge).length,
    0,
    "self-test failed: closure predicate flagged an externally-published target",
  );
  assert.equal(
    closureGaps(probeTrain, new Set(), [["probe-train-crate", "probe-train-crate"]]).length,
    0,
    "self-test failed: closure predicate flagged an in-train edge",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.publish-train-closure",
      trainCrateCount: trainCrates.length,
      publishOrderCount: publishOrder.length,
      externallyPublished: [...externalCrates],
      shippedPathDepEdges: edges.length,
      closureViolations: 0,
      publishOrderTopologicallyValid: true,
      publishOrderIsCanonical: true,
      edgeSetSha256,
      lockStep: true,
    },
    null,
    2,
  )}\n`,
);
