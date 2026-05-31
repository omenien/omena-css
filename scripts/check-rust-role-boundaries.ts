import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/role-boundaries
 *
 * The role manifest is the single source of truth for each workspace member's
 * STRUCTURAL role (orthogonal to master-plan §4 MATURITY tier). It lives in each
 * crate's `[package.metadata.omena]` table (role + optional pillar), so the role
 * travels with the crate through any future extraction. This gate asserts the
 * manifest is complete and self-consistent, and that the role-dependency edges
 * obey the structural invariants.
 *
 * Roles:
 *   R1 reusable building block | R2 composed engine library | U umbrella facade
 *   P product surface | I internal/dev (publish=false) | S support/infrastructure
 *
 * R1-vs-R2 is a COMPUTED predicate, not a hand judgment: a crate is R2 iff it is
 * itself one of the pinned engine hubs OR its transitive internal-dep closure
 * reaches >= R2_REACH_THRESHOLD of them. The gate recomputes it and asserts the
 * declared role matches — a mis-tagged role reds CI.
 *
 * Edge invariants enforced here:
 *   - no R1 -> R2 edge (a building block must not depend on the engine)
 *   - [I] is unreachable from any published crate (publish=false crates are sinks)
 * The [P] -> engine facade dep-set is intentionally NOT asserted here; it is
 * delegated to the existing standalone product-boundary gates (per plan §5.1).
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const VALID_ROLES = new Set(["R1", "R2", "U", "P", "I", "S"]);
const PUBLISHED_ROLES = new Set(["R1", "R2", "U", "P", "S"]);

// Pinned engine-hub family for the R1/R2 computed predicate. These are the
// composed-engine crates; a crate that reaches >= threshold of them composes the
// engine and is therefore R2. Recorded here as the predicate anchor (plan §5.1).
const R2_FAMILY = new Set([
  "omena-semantic",
  "omena-bridge",
  "omena-query",
  "omena-query-core",
  "omena-query-checker-orchestrator",
  "omena-query-transform-runner",
  "omena-checker",
  "omena-transform-passes",
  "omena-transform-bundle",
  "omena-transform-target",
  "omena-transform-print",
  "omena-transform-egg",
]);
const R2_REACH_THRESHOLD = 2;

interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
}
interface CargoPackage {
  readonly name: string;
  readonly dependencies: readonly CargoDependency[];
  readonly metadata?: { readonly omena?: { readonly role?: string; readonly pillar?: string } };
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const memberNames = new Set(metadata.packages.map((pkg) => pkg.name));
const roleOf = new Map<string, string>();
const internalDeps = new Map<string, Set<string>>();

// (1) Completeness + valid role + valid pillar: every member declares exactly one
//     recognised role; a pillar, if present, must be the only supported value.
for (const pkg of metadata.packages) {
  const role = pkg.metadata?.omena?.role;
  assert.ok(
    role !== undefined,
    `workspace member ${pkg.name} is missing [package.metadata.omena].role`,
  );
  assert.ok(
    VALID_ROLES.has(role),
    `workspace member ${pkg.name} has invalid role "${role}" (expected one of ${[...VALID_ROLES].join(", ")})`,
  );
  roleOf.set(pkg.name, role);

  const pillar = pkg.metadata?.omena?.pillar;
  if (pillar !== undefined) {
    assert.equal(
      pillar,
      "theoretical-rigor",
      `workspace member ${pkg.name} has unsupported pillar "${pillar}" (only "theoretical-rigor" is defined)`,
    );
  }

  const deps = new Set<string>();
  for (const dep of pkg.dependencies) {
    if (dep.kind !== "dev" && memberNames.has(dep.name)) {
      deps.add(dep.name);
    }
  }
  internalDeps.set(pkg.name, deps);
}

function transitiveInternalClosure(crate: string): Set<string> {
  const seen = new Set<string>();
  const stack = [...(internalDeps.get(crate) ?? [])];
  while (stack.length > 0) {
    const current = stack.pop()!;
    if (seen.has(current)) {
      continue;
    }
    seen.add(current);
    for (const next of internalDeps.get(current) ?? []) {
      stack.push(next);
    }
  }
  return seen;
}

/** Shared R1/R2 predicate (also exercised by the self-test below). */
function computesAsR2(crate: string, family: ReadonlySet<string>, closure: ReadonlySet<string>): boolean {
  if (family.has(crate)) {
    return true;
  }
  let reached = 0;
  for (const hub of family) {
    if (closure.has(hub)) {
      reached += 1;
    }
  }
  return reached >= R2_REACH_THRESHOLD;
}

// (2) R1/R2 computed-predicate cross-check.
const predicateViolations: string[] = [];
for (const [crate, role] of roleOf) {
  if (role !== "R1" && role !== "R2") {
    continue;
  }
  const computedR2 = computesAsR2(crate, R2_FAMILY, transitiveInternalClosure(crate));
  if (role === "R2" && !computedR2) {
    predicateViolations.push(`${crate} is declared R2 but the predicate computes R1 (it does not compose the engine)`);
  }
  if (role === "R1" && computedR2) {
    predicateViolations.push(`${crate} is declared R1 but the predicate computes R2 (it composes the engine) — declare it R2`);
  }
}
assert.equal(
  predicateViolations.length,
  0,
  `R1/R2 role declarations disagree with the computed predicate:\n  ${predicateViolations.join("\n  ")}`,
);

// (3) Edge invariant: no R1 -> R2.
const r1ToR2: string[] = [];
for (const [crate, role] of roleOf) {
  if (role !== "R1") {
    continue;
  }
  for (const dep of internalDeps.get(crate) ?? []) {
    if (roleOf.get(dep) === "R2") {
      r1ToR2.push(`${crate} (R1) -> ${dep} (R2)`);
    }
  }
}
assert.equal(
  r1ToR2.length,
  0,
  `R1 building blocks must not depend on R2 engine libraries:\n  ${r1ToR2.join("\n  ")}`,
);

// (4) Edge invariant: [I] is unreachable from any published crate.
const publishedToInternal: string[] = [];
for (const [crate, role] of roleOf) {
  if (!PUBLISHED_ROLES.has(role)) {
    continue;
  }
  for (const dep of internalDeps.get(crate) ?? []) {
    if (roleOf.get(dep) === "I") {
      publishedToInternal.push(`${crate} (${role}) -> ${dep} (I)`);
    }
  }
}
assert.equal(
  publishedToInternal.length,
  0,
  `published crates must not depend on internal [I] crates (publish=false sinks):\n  ${publishedToInternal.join("\n  ")}`,
);

// (5) Self-test of the predicate and edge logic on synthetic inputs.
{
  const probeFamily = new Set(["hub-a", "hub-b"]);
  assert.ok(
    computesAsR2("composes", probeFamily, new Set(["hub-a", "hub-b"])),
    "self-test: a crate reaching >= threshold hubs must compute R2",
  );
  assert.ok(
    !computesAsR2("leaf", probeFamily, new Set(["hub-a"])),
    "self-test: a crate reaching < threshold hubs must compute R1",
  );
  assert.ok(
    computesAsR2("hub-a", probeFamily, new Set()),
    "self-test: a hub crate must compute R2 by membership",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.role-boundaries",
      members: metadata.packages.length,
      roleCounts: [...roleOf.values()].reduce<Record<string, number>>((acc, role) => {
        acc[role] = (acc[role] ?? 0) + 1;
        return acc;
      }, {}),
      r2ReachThreshold: R2_REACH_THRESHOLD,
      predicateViolations: 0,
      r1ToR2Edges: 0,
      publishedToInternalEdges: 0,
    },
    null,
    2,
  )}\n`,
);
