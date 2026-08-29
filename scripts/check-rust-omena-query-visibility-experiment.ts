import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type AuthoredClass = "c_internal_only" | "c_tests_only";
type Visibility = "crate" | "public";
type Outcome =
  | "compiledCrateVisible"
  | "integrationConsumerRequired"
  | "warningFreeDeletionRequired"
  | "testOnlyExcluded";

interface VisibilityRow {
  readonly name: string;
  readonly originCrate: string;
  readonly authoredClass: AuthoredClass;
  readonly visibility: Visibility;
  readonly outcome: Outcome;
}

interface VisibilityExperiment {
  readonly schemaVersion: "0";
  readonly product: "omena-query.visibility-experiment";
  readonly authoredCandidateCount: 391;
  readonly commands: {
    readonly compile: string;
    readonly warningPolicy: string;
  };
  readonly outcomeCounts: Readonly<Record<Outcome, number>>;
  readonly rows: readonly VisibilityRow[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tablePath = path.join(repoRoot, "rust/omena-query-visibility-experiment.json");
const sourcePath = path.join(repoRoot, "rust/crates/omena-query/src/lib.rs");
const integrationTestPath = path.join(
  repoRoot,
  "rust/crates/omena-query/tests/sdk_workflow_contract.rs",
);
const table = JSON.parse(readFileSync(tablePath, "utf8")) as VisibilityExperiment;
const source = readFileSync(sourcePath, "utf8");
const integrationTest = readFileSync(integrationTestPath, "utf8");

const integrationConsumerNames = new Set([
  "OmenaSdkBuildVerificationProfileV0",
  "OmenaSdkBuildVerificationReasonV0",
  "OmenaSdkExplainPositionV0",
  "OmenaSdkResponsePartitionV0",
  "OmenaSdkSnapshotResponseV0",
  "execute_omena_sdk_diagnostics_debug_workflow",
  "execute_omena_sdk_diagnostics_workflow",
  "omena_error_from_boundary_encoding",
]);

validateExperiment(table, scanFacadeVisibility(source), integrationTest);
runValidatorSelftests(table, source, integrationTest);

runCargo([
  "check",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-query",
  "--all-targets",
  "--all-features",
]);
runCargo([
  "clippy",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-query",
  "--all-targets",
  "--all-features",
  "--no-deps",
  "--",
  "-D",
  "warnings",
]);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-query.visibility-experiment",
      authoredCandidateCount: table.rows.length,
      outcomeCounts: table.outcomeCounts,
      compile: "green",
      warningPolicy: "green",
      selftestMutationCount: 2,
    },
    null,
    2,
  )}\n`,
);

function validateExperiment(
  candidate: VisibilityExperiment,
  observedVisibility: ReadonlyMap<string, Visibility>,
  candidateIntegrationTest: string,
): void {
  assert.equal(candidate.schemaVersion, "0");
  assert.equal(candidate.product, "omena-query.visibility-experiment");
  assert.equal(candidate.authoredCandidateCount, 391);
  assert.equal(candidate.rows.length, candidate.authoredCandidateCount);
  assert.equal(new Set(candidate.rows.map((row) => row.name)).size, candidate.rows.length);
  assert.deepEqual(candidate.outcomeCounts, {
    compiledCrateVisible: 279,
    integrationConsumerRequired: 8,
    warningFreeDeletionRequired: 86,
    testOnlyExcluded: 18,
  });
  assert.equal(
    candidate.commands.compile,
    "cargo check --manifest-path rust/Cargo.toml -p omena-query --all-targets --all-features",
  );
  assert.equal(
    candidate.commands.warningPolicy,
    "cargo clippy --manifest-path rust/Cargo.toml -p omena-query --all-targets --all-features --no-deps -- -D warnings",
  );

  const measuredCounts = new Map<Outcome, number>();
  const observedIntegrationNames = new Set<string>();
  for (const row of candidate.rows) {
    assert.ok(row.name.length > 0, "visibility row names must be non-empty");
    assert.ok(row.originCrate.length > 0, `${row.name} must name its origin crate`);
    const observed = observedVisibility.get(row.name);
    assert.equal(observed, row.visibility, `${row.name} visibility drifted`);
    measuredCounts.set(row.outcome, (measuredCounts.get(row.outcome) ?? 0) + 1);

    if (row.outcome === "compiledCrateVisible") {
      assert.equal(row.authoredClass, "c_internal_only");
      assert.equal(row.visibility, "crate");
    } else if (row.outcome === "testOnlyExcluded") {
      assert.equal(row.authoredClass, "c_tests_only");
      assert.equal(row.visibility, "public");
    } else {
      assert.equal(row.authoredClass, "c_internal_only");
      assert.equal(row.visibility, "public");
    }

    if (row.outcome === "integrationConsumerRequired") {
      observedIntegrationNames.add(row.name);
      assert.match(
        candidateIntegrationTest,
        new RegExp(`\\b${escapeRegExp(row.name)}\\b`, "u"),
        `${row.name} is not consumed by the integration contract test`,
      );
    }
  }

  for (const [outcome, expectedCount] of Object.entries(candidate.outcomeCounts) as readonly [
    Outcome,
    number,
  ][]) {
    assert.equal(measuredCounts.get(outcome), expectedCount, `${outcome} count drifted`);
  }
  assert.deepEqual(observedIntegrationNames, integrationConsumerNames);
}

function runValidatorSelftests(
  candidate: VisibilityExperiment,
  candidateSource: string,
  candidateIntegrationTest: string,
): void {
  const observed = scanFacadeVisibility(candidateSource);
  const compiled = candidate.rows.find((row) => row.outcome === "compiledCrateVisible");
  assert.ok(compiled);
  const visibilityMutation = new Map(observed);
  visibilityMutation.set(compiled.name, "public");
  assert.throws(
    () => validateExperiment(candidate, visibilityMutation, candidateIntegrationTest),
    /visibility drifted/u,
  );

  assert.throws(
    () =>
      validateExperiment(
        { ...candidate, rows: candidate.rows.slice(1) } as VisibilityExperiment,
        observed,
        candidateIntegrationTest,
      ),
    /391/u,
  );
}

function scanFacadeVisibility(candidateSource: string): ReadonlyMap<string, Visibility> {
  const visibility = new Map<string, Visibility>();
  const groupedUsePattern =
    /((?:#\[[^\]]*\]\s*)*)pub(?:(\(crate\)))?\s+use\s+[A-Za-z0-9_:]+::\{([\s\S]*?)\};/gu;
  for (const match of candidateSource.matchAll(groupedUsePattern)) {
    const observed = match[2] === "(crate)" ? "crate" : "public";
    for (const item of (match[3] ?? "").split(",")) {
      const name = exportedName(item.trim());
      if (name !== null) {
        visibility.set(name, observed);
      }
    }
  }

  const declarationPattern =
    /\bpub(?:(\(crate\)))?\s+(?:async\s+)?(?:struct|enum|fn|type|const|static|trait)\s+([A-Za-z0-9_]+)/gu;
  for (const match of candidateSource.matchAll(declarationPattern)) {
    const name = match[2];
    assert.ok(name);
    visibility.set(name, match[1] === "(crate)" ? "crate" : "public");
  }

  const singleUsePattern = /\bpub(?:(\(crate\)))?\s+use\s+[A-Za-z0-9_:]+::([A-Za-z0-9_]+)\s*;/gu;
  for (const match of candidateSource.matchAll(singleUsePattern)) {
    const name = match[2];
    assert.ok(name);
    visibility.set(name, match[1] === "(crate)" ? "crate" : "public");
  }
  return visibility;
}

function exportedName(item: string): string | null {
  if (item.length === 0) {
    return null;
  }
  return (
    item.match(/\bas\s+([A-Za-z0-9_]+)$/u)?.[1] ?? item.match(/([A-Za-z0-9_]+)$/u)?.[1] ?? null
  );
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function runCargo(args: readonly string[]): void {
  execFileSync("cargo", args, { cwd: repoRoot, stdio: "inherit" });
}
