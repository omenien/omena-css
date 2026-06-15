import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/layer-dependency-exceptions
 *
 * This gate covers the narrower architectural edge that rust/role-boundaries
 * deliberately does not model with exception metadata: lower/theory substrates
 * must not depend on public facade crates unless the edge has an explicit
 * rationale and retirement path.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const FACADE_CRATES = new Set([
  "omena-query",
  "omena-query-checker-orchestrator",
  "omena-query-transform-runner",
]);

const FACADE_WATCHED_PILLARS = new Set(["theoretical-rigor"]);

const REQUIRED_WATCHED_CRATES = [
  {
    name: "omena-streaming-ifds",
    pillar: "theoretical-rigor",
    reason:
      "Keep streaming IFDS facade-dependency regressions visible after retiring its omena-query exception.",
  },
] as const;

const ALLOWED_EXCEPTIONS = [] as const;

interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
}

interface CargoPackage {
  readonly name: string;
  readonly dependencies: readonly CargoDependency[];
  readonly metadata?: {
    readonly omena?: {
      readonly role?: string;
      readonly pillar?: string;
    };
  };
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const packagesByName = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));
const memberNames = new Set(packagesByName.keys());
const roleOf = new Map<string, string>();
const pillarOf = new Map<string, string | undefined>();
const internalDeps = new Map<string, Set<string>>();

for (const pkg of metadata.packages) {
  const role = pkg.metadata?.omena?.role;
  assert.ok(role !== undefined, `workspace member ${pkg.name} is missing omena role metadata`);
  roleOf.set(pkg.name, role);
  pillarOf.set(pkg.name, pkg.metadata?.omena?.pillar);

  const deps = new Set<string>();
  for (const dep of pkg.dependencies) {
    if (dep.kind !== "dev" && memberNames.has(dep.name)) {
      deps.add(dep.name);
    }
  }
  internalDeps.set(pkg.name, deps);
}

for (const watchedCrate of REQUIRED_WATCHED_CRATES) {
  const actualPillar = pillarOf.get(watchedCrate.name);
  assert.equal(
    actualPillar,
    watchedCrate.pillar,
    `${watchedCrate.name} must keep pillar=${watchedCrate.pillar}: ${watchedCrate.reason}`,
  );
}

function edgeKey(from: string, to: string): string {
  return `${from}->${to}`;
}

const allowedByEdge = new Map<string, (typeof ALLOWED_EXCEPTIONS)[number]>();
for (const exception of ALLOWED_EXCEPTIONS) {
  assert.ok(
    exception.reason.trim().length > 0,
    `${edgeKey(exception.from, exception.to)} needs a reason`,
  );
  assert.ok(
    exception.retirementPath.trim().length > 0,
    `${edgeKey(exception.from, exception.to)} needs a retirement path`,
  );
  assert.ok(
    packagesByName.has(exception.from),
    `layer dependency exception source crate does not exist: ${exception.from}`,
  );
  assert.ok(
    packagesByName.has(exception.to),
    `layer dependency exception target crate does not exist: ${exception.to}`,
  );
  assert.ok(
    !allowedByEdge.has(edgeKey(exception.from, exception.to)),
    `duplicate layer dependency exception: ${edgeKey(exception.from, exception.to)}`,
  );
  allowedByEdge.set(edgeKey(exception.from, exception.to), exception);
}

const violations: string[] = [];
const observedExceptions = new Set<string>();

for (const [crate, deps] of internalDeps) {
  const role = roleOf.get(crate);
  const pillar = pillarOf.get(crate);

  for (const dep of deps) {
    const depRole = roleOf.get(dep);
    const isR1ToR2 = role === "R1" && depRole === "R2";
    const isWatchedPillarToFacade =
      pillar !== undefined && FACADE_WATCHED_PILLARS.has(pillar) && FACADE_CRATES.has(dep);
    if (!isR1ToR2 && !isWatchedPillarToFacade) {
      continue;
    }

    const key = edgeKey(crate, dep);
    if (allowedByEdge.has(key)) {
      observedExceptions.add(key);
      continue;
    }

    const reasons = [
      isR1ToR2 ? "R1 building block depends on R2 facade/engine" : undefined,
      isWatchedPillarToFacade ? `${pillar} crate depends on facade crate` : undefined,
    ].filter(Boolean);
    violations.push(`${key}: ${reasons.join("; ")}`);
  }
}

assert.equal(
  violations.length,
  0,
  `unapproved Rust layer dependency exceptions:\n  ${violations.join("\n  ")}`,
);

const staleExceptions = [...allowedByEdge.keys()].filter((key) => !observedExceptions.has(key));
assert.equal(
  staleExceptions.length,
  0,
  `stale Rust layer dependency exceptions no longer matching Cargo metadata:\n  ${staleExceptions.join("\n  ")}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.layer-dependency-exceptions",
      members: metadata.packages.length,
      facadeCrates: [...FACADE_CRATES].toSorted(),
      facadeWatchedPillars: [...FACADE_WATCHED_PILLARS].toSorted(),
      requiredWatchedCrates: REQUIRED_WATCHED_CRATES.map((crate) => crate.name).toSorted(),
      allowedExceptionCount: ALLOWED_EXCEPTIONS.length,
      observedExceptions: [...observedExceptions].toSorted(),
      unapprovedExceptions: 0,
      staleExceptions: 0,
    },
    null,
    2,
  )}\n`,
);
