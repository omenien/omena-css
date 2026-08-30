import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type AuthoredClass = "c_compatibility_promise" | "c_internal_only" | "c_tests_only";
type Visibility = "crate" | "public";
type Outcome =
  | "compiledCrateVisible"
  | "compatibilityPromiseRestored"
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
  readonly defaultFeatureNarrowing: {
    readonly feature: "transform-catalog-trace";
    readonly rowCount: 44;
    readonly rows: readonly {
      readonly name: string;
      readonly defaultVisibility: "crate";
      readonly allFeaturesVisibility: "public";
      readonly releaseClass: "pre1MinorBreaking";
    }[];
  };
  readonly rows: readonly VisibilityRow[];
}

interface DefaultWarningDispositions {
  readonly schemaVersion: "0";
  readonly product: "omena-query.default-feature-warning-dispositions";
  readonly feature: "transform-catalog-trace";
  readonly defaultWarningFloor: 0;
  readonly measuredForcedWarningCount: 83;
  readonly newlyDispositionedWarningCount: 60;
  readonly preexistingAllowedWarningCount: 23;
  readonly dispositions: {
    readonly retainedForFeatureEnabledQuerySurface: readonly string[];
    readonly preexistingTargetedAllowance: readonly string[];
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tablePath = path.join(repoRoot, "rust/omena-query-visibility-experiment.json");
const warningDispositionPath = path.join(
  repoRoot,
  "rust/omena-query-default-warning-dispositions.json",
);
const sourcePath = path.join(repoRoot, "rust/crates/omena-query/src/lib.rs");
const integrationTestPath = path.join(
  repoRoot,
  "rust/crates/omena-query/tests/sdk_workflow_contract.rs",
);
const table = JSON.parse(readFileSync(tablePath, "utf8")) as VisibilityExperiment;
const warningDispositions = JSON.parse(
  readFileSync(warningDispositionPath, "utf8"),
) as DefaultWarningDispositions;
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

const defaultVisibility = scanFacadeVisibility(source, new Set());
const allFeaturesVisibility = scanFacadeVisibility(source, cargoFeatureNames());
validateExperiment(table, allFeaturesVisibility, integrationTest);
validateDefaultFeatureNarrowing(table, defaultVisibility, allFeaturesVisibility);
const forcedDefaultWarnings = measureForcedDefaultFeatureWarnings();
validateDefaultWarningDispositions(warningDispositions, forcedDefaultWarnings);
runValidatorSelftests(table, source, integrationTest);
runDefaultWarningDispositionSelftest(warningDispositions, forcedDefaultWarnings);

if (process.argv.includes("--inject-remove-warning-disposition")) {
  validateDefaultWarningDispositions(
    {
      ...warningDispositions,
      dispositions: {
        ...warningDispositions.dispositions,
        retainedForFeatureEnabledQuerySurface:
          warningDispositions.dispositions.retainedForFeatureEnabledQuerySurface.slice(1),
      },
    },
    forcedDefaultWarnings,
  );
}

assert.match(
  source,
  /#!\[cfg_attr\(not\(feature = "transform-catalog-trace"\), allow\(dead_code\)\)\]/,
  "default-feature warning disposition must be explicit and feature-scoped",
);
runDefaultFeatureWarningFloor();

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
      defaultFeatureNarrowingCount: table.defaultFeatureNarrowing.rows.length,
      defaultWarningFloor: warningDispositions.defaultWarningFloor,
      newlyDispositionedDefaultWarningCount: warningDispositions.newlyDispositionedWarningCount,
      preexistingAllowedWarningCount: warningDispositions.preexistingAllowedWarningCount,
      forcedDefaultWarningCensusCount: forcedDefaultWarnings.length,
      compile: "green",
      warningPolicy: "green",
      selftestMutationCount: 4,
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
    compiledCrateVisible: 276,
    compatibilityPromiseRestored: 3,
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
    } else if (row.outcome === "compatibilityPromiseRestored") {
      assert.equal(row.authoredClass, "c_compatibility_promise");
      assert.equal(row.visibility, "public");
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

function validateDefaultFeatureNarrowing(
  candidate: VisibilityExperiment,
  defaultVisibility: ReadonlyMap<string, Visibility>,
  allFeaturesVisibility: ReadonlyMap<string, Visibility>,
): void {
  assert.equal(candidate.defaultFeatureNarrowing.feature, "transform-catalog-trace");
  assert.equal(candidate.defaultFeatureNarrowing.rowCount, 44);
  assert.equal(candidate.defaultFeatureNarrowing.rows.length, 44);
  assert.equal(
    new Set(candidate.defaultFeatureNarrowing.rows.map((row) => row.name)).size,
    candidate.defaultFeatureNarrowing.rows.length,
    "default-feature narrowing rows must be unique",
  );
  const measured = [...allFeaturesVisibility.entries()]
    .filter(
      ([name, visibility]) => visibility === "public" && defaultVisibility.get(name) === "crate",
    )
    .map(([name]) => name)
    .toSorted();
  assert.deepEqual(
    measured,
    candidate.defaultFeatureNarrowing.rows.map((row) => row.name).toSorted(),
    "default-private/all-features-public surface set drifted",
  );
  const authoredNames = new Set(candidate.rows.map((row) => row.name));
  for (const row of candidate.defaultFeatureNarrowing.rows) {
    assert.equal(row.defaultVisibility, "crate");
    assert.equal(row.allFeaturesVisibility, "public");
    assert.equal(row.releaseClass, "pre1MinorBreaking");
    assert.ok(authoredNames.has(row.name), `${row.name} lacks an authored visibility row`);
  }
}

function runValidatorSelftests(
  candidate: VisibilityExperiment,
  candidateSource: string,
  candidateIntegrationTest: string,
): void {
  const observed = scanFacadeVisibility(candidateSource, cargoFeatureNames());
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

  const cfgMutation = candidateSource.replace(
    '#[cfg(not(feature = "transform-catalog-trace"))]',
    '#[cfg(feature = "transform-catalog-trace")]',
  );
  assert.throws(
    () =>
      validateDefaultFeatureNarrowing(
        candidate,
        scanFacadeVisibility(cfgMutation, new Set()),
        scanFacadeVisibility(cfgMutation, cargoFeatureNames()),
      ),
    /default-private\/all-features-public/u,
    "cfg-aware visibility selftest must reject a missing default-private branch",
  );
}

function scanFacadeVisibility(
  candidateSource: string,
  enabledFeatures: ReadonlySet<string>,
): ReadonlyMap<string, Visibility> {
  const visibility = new Map<string, Visibility>();
  const groupedUsePattern =
    /((?:#\[[^\]]*\]\s*)*)pub(?:(\(crate\)))?\s+use\s+[A-Za-z0-9_:]+::\{([\s\S]*?)\};/gu;
  for (const match of candidateSource.matchAll(groupedUsePattern)) {
    if (!cfgAttributesAreActive(match[1] ?? "", enabledFeatures)) continue;
    const observed = match[2] === "(crate)" ? "crate" : "public";
    for (const item of (match[3] ?? "").split(",")) {
      const name = exportedName(item.trim());
      if (name !== null) {
        visibility.set(name, observed);
      }
    }
  }

  const declarationPattern =
    /((?:#\[[^\]]*\]\s*)*)\bpub(?:(\(crate\)))?\s+(?:async\s+)?(?:struct|enum|fn|type|const|static|trait)\s+([A-Za-z0-9_]+)/gu;
  for (const match of candidateSource.matchAll(declarationPattern)) {
    if (!cfgAttributesAreActive(match[1] ?? "", enabledFeatures)) continue;
    const name = match[3];
    assert.ok(name);
    visibility.set(name, match[2] === "(crate)" ? "crate" : "public");
  }

  const singleUsePattern =
    /((?:#\[[^\]]*\]\s*)*)\bpub(?:(\(crate\)))?\s+use\s+[A-Za-z0-9_:]+::([A-Za-z0-9_]+)\s*;/gu;
  for (const match of candidateSource.matchAll(singleUsePattern)) {
    if (!cfgAttributesAreActive(match[1] ?? "", enabledFeatures)) continue;
    const name = match[3];
    assert.ok(name);
    visibility.set(name, match[2] === "(crate)" ? "crate" : "public");
  }
  return visibility;
}

function cargoFeatureNames(): ReadonlySet<string> {
  const manifest = readFileSync(path.join(repoRoot, "rust/crates/omena-query/Cargo.toml"), "utf8");
  const section = manifest.match(/\[features\]\n([\s\S]*?)(?=\n\[)/u)?.[1] ?? "";
  return new Set(
    [...section.matchAll(/^([A-Za-z0-9_-]+)\s*=/gmu)]
      .map((match) => match[1]!)
      .filter((feature) => feature !== "default"),
  );
}

function cfgAttributesAreActive(attributes: string, enabledFeatures: ReadonlySet<string>): boolean {
  const expressions = [...attributes.matchAll(/#\[cfg\(([^\]]+)\)\]/gu)].map((match) => match[1]!);
  return expressions.every((expression) => evaluateCfg(expression, enabledFeatures));
}

function evaluateCfg(expression: string, enabledFeatures: ReadonlySet<string>): boolean {
  const value = expression.trim();
  const feature = /^feature\s*=\s*"([^"]+)"$/u.exec(value);
  if (feature) return enabledFeatures.has(feature[1]!);
  if (value === "test") return false;
  for (const [operator, predicate] of [
    ["not", (values: readonly boolean[]) => !values[0]],
    ["all", (values: readonly boolean[]) => values.every(Boolean)],
    ["any", (values: readonly boolean[]) => values.some(Boolean)],
  ] as const) {
    if (value.startsWith(`${operator}(`) && value.endsWith(")")) {
      const operands = splitCfgOperands(value.slice(operator.length + 1, -1));
      if (operator === "not") assert.equal(operands.length, 1, "cfg(not()) requires one operand");
      return predicate(operands.map((operand) => evaluateCfg(operand, enabledFeatures)));
    }
  }
  throw new Error(`unsupported cfg expression in query visibility scanner: ${value}`);
}

function splitCfgOperands(value: string): readonly string[] {
  const operands: string[] = [];
  let start = 0;
  let depth = 0;
  for (let index = 0; index < value.length; index += 1) {
    if (value[index] === "(") depth += 1;
    if (value[index] === ")") depth -= 1;
    if (value[index] === "," && depth === 0) {
      operands.push(value.slice(start, index).trim());
      start = index + 1;
    }
  }
  operands.push(value.slice(start).trim());
  return operands.filter(Boolean);
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

function measureForcedDefaultFeatureWarnings(): readonly string[] {
  const run = spawnSync(
    "cargo",
    ["check", "--manifest-path", "rust/Cargo.toml", "-p", "omena-query", "--message-format=json"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, CARGO_TERM_COLOR: "never", RUSTFLAGS: "--force-warn=dead-code" },
    },
  );
  assert.equal(
    run.status,
    0,
    `forced default-feature warning census failed\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`,
  );
  const warnings: string[] = [];
  for (const line of run.stdout.split("\n")) {
    if (line.length === 0) continue;
    const message = JSON.parse(line) as {
      readonly reason?: string;
      readonly target?: { readonly name?: string };
      readonly message?: {
        readonly level?: string;
        readonly message?: string;
        readonly code?: { readonly code?: string } | null;
        readonly spans?: readonly { readonly file_name?: string; readonly is_primary?: boolean }[];
      };
    };
    if (
      message.reason !== "compiler-message" ||
      message.target?.name !== "omena_query" ||
      message.message?.level !== "warning" ||
      message.message.code?.code !== "dead_code"
    ) {
      continue;
    }
    const primary = message.message.spans?.find((span) => span.is_primary);
    assert.ok(primary?.file_name);
    assert.ok(message.message.message);
    warnings.push(
      `${primary.file_name.replace(/^crates\//u, "rust/crates/")}::${message.message.message}`,
    );
  }
  return warnings.toSorted();
}

function validateDefaultWarningDispositions(
  candidate: DefaultWarningDispositions,
  measured: readonly string[],
): void {
  assert.equal(candidate.schemaVersion, "0");
  assert.equal(candidate.product, "omena-query.default-feature-warning-dispositions");
  assert.equal(candidate.feature, "transform-catalog-trace");
  assert.equal(candidate.defaultWarningFloor, 0);
  assert.equal(candidate.measuredForcedWarningCount, 83);
  assert.equal(candidate.newlyDispositionedWarningCount, 60);
  assert.equal(candidate.preexistingAllowedWarningCount, 23);
  assert.equal(
    candidate.dispositions.retainedForFeatureEnabledQuerySurface.length,
    candidate.newlyDispositionedWarningCount,
  );
  assert.equal(
    candidate.dispositions.preexistingTargetedAllowance.length,
    candidate.preexistingAllowedWarningCount,
  );
  const declared = [
    ...candidate.dispositions.retainedForFeatureEnabledQuerySurface,
    ...candidate.dispositions.preexistingTargetedAllowance,
  ].toSorted();
  assert.equal(declared.length, candidate.measuredForcedWarningCount);
  assert.deepEqual(measured, declared, "default-feature dead-code warning disposition set drifted");
}

function runDefaultWarningDispositionSelftest(
  candidate: DefaultWarningDispositions,
  measured: readonly string[],
): void {
  assert.throws(
    () =>
      validateDefaultWarningDispositions(
        {
          ...candidate,
          dispositions: {
            ...candidate.dispositions,
            retainedForFeatureEnabledQuerySurface:
              candidate.dispositions.retainedForFeatureEnabledQuerySurface.slice(1),
          },
        },
        measured,
      ),
    /60|disposition set drifted/u,
  );
}

function runDefaultFeatureWarningFloor(): void {
  const run = spawnSync(
    "cargo",
    ["check", "--manifest-path", "rust/Cargo.toml", "-p", "omena-query"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, CARGO_TERM_COLOR: "never", RUSTFLAGS: "-Dwarnings" },
    },
  );
  assert.equal(
    run.status,
    0,
    `default-feature warning floor regressed\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`,
  );
}

function runCargo(args: readonly string[]): void {
  execFileSync("cargo", args, { cwd: repoRoot, stdio: "inherit" });
}
