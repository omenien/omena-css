import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
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
  readonly reexportBoundaryReconstruction: {
    readonly publicApiTool: "cargo-public-api 0.52.0";
    readonly pickupRevision: "1d97626b3d3bd335945dcb43b261821aa91279a4";
    readonly explicitReexportRevision: "8df0ea1f9485fc6cecfd2f236543ecb9473374e2";
    readonly expectedChangedPaths: readonly [
      "rust/crates/omena-query/src/boundary.rs",
      "rust/crates/omena-query/src/style.rs",
    ];
    readonly authoredEntryGate: {
      readonly facadeNameCount: 914;
      readonly defaultRootNameCount: 859;
      readonly allFeaturesOnlyRootNameCount: 61;
      readonly renamedOnlyCandidateCount: 67;
      readonly renamedOnlyCandidateListSha256: "sha256:330931f068fe11173c0c264322606cd5aabf8f7f54677f87477d42cebb4cec2d";
      readonly renameOnlyCriterion: "exported alias differs from origin leaf name";
    };
    readonly pickupEntryGate: {
      readonly namedReexportCount: 923;
      readonly publicTypeAliasCount: 1;
      readonly facadeNameCount: 924;
      readonly defaultRootNameCount: 869;
      readonly allFeaturesRootNameCount: 930;
      readonly allFeaturesOnlyRootNameCount: 61;
      readonly semverIntentCount: 0;
    };
    readonly explicitReexportReplay: {
      readonly pickupWildcardReexportCount: 15;
      readonly explicitReexportWildcardReexportCount: 0;
      readonly defaultFeatures: ReexportReplayPlane;
      readonly allFeatures: ReexportReplayPlane;
    };
    readonly currentSurfaceAccounting: {
      readonly defaultRootNameCount: 573;
      readonly allFeaturesRootNameCount: 658;
      readonly allFeaturesOnlyRootNameCount: 85;
      readonly defaultRootNameLossFromPickup: 296;
      readonly allFeaturesRootNameLossFromPickup: 272;
      readonly postPickupPublicAdditionCount: 7;
      readonly postPickupPublicAdditionNames: readonly string[];
      readonly compiledVisibilityReductionCount: 276;
      readonly compiledVisibilityNamesAbsentFromPickupDefaultCount: 20;
      readonly defaultOnlyNarrowingCount: 44;
      readonly retiredRefreshSurfaceCount: 3;
      readonly restoredCompatibilityPromiseCount: 3;
      readonly defaultLossResidueCount: 20;
      readonly defaultLossResidueEquation: "44 - 20 + 3 - 7 = 20";
      readonly compiledVisibilityNamesAbsentFromPickupDefault: readonly string[];
      readonly retiredRefreshSurfaceNames: readonly string[];
    };
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

interface ReexportReplayPlane {
  readonly pickupOutputSha256: string;
  readonly explicitReexportOutputSha256: string;
  readonly pickupRootNameCount: number;
  readonly explicitReexportRootNameCount: number;
  readonly addedRootNameCount: 0;
  readonly removedRootNameCount: 0;
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
const defaultPublicApiSnapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-query/tests/snapshots/public-api.txt",
);
const allFeaturesPublicApiSnapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-query/tests/snapshots/public-api-all-features.txt",
);
const semverIntentPath = path.join(repoRoot, "rust/omena-rust-semver-intent.json");
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
validateReexportBoundaryReconstruction(table);
const replayedReexportBoundary = process.argv.includes("--replay-reexport-boundary");
if (replayedReexportBoundary) {
  replayReexportBoundary(table);
}
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
      reexportBoundaryReconstruction: replayedReexportBoundary ? "replayed" : "validated",
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

function validateReexportBoundaryReconstruction(candidate: VisibilityExperiment): void {
  const reconstruction = candidate.reexportBoundaryReconstruction;
  assert.equal(reconstruction.publicApiTool, "cargo-public-api 0.52.0");
  const parentRevision = gitText([
    "rev-parse",
    `${reconstruction.explicitReexportRevision}^`,
  ]).trim();
  assert.equal(parentRevision, reconstruction.pickupRevision);
  const changedPaths = gitText([
    "diff-tree",
    "--no-commit-id",
    "--name-only",
    "-r",
    reconstruction.explicitReexportRevision,
  ])
    .trim()
    .split("\n")
    .filter(Boolean)
    .toSorted();
  assert.deepEqual(changedPaths, [...reconstruction.expectedChangedPaths].toSorted());

  assert.deepEqual(reconstruction.authoredEntryGate, {
    facadeNameCount: 914,
    defaultRootNameCount: 859,
    allFeaturesOnlyRootNameCount: 61,
    renamedOnlyCandidateCount: 67,
    renamedOnlyCandidateListSha256:
      "sha256:330931f068fe11173c0c264322606cd5aabf8f7f54677f87477d42cebb4cec2d",
    renameOnlyCriterion: "exported alias differs from origin leaf name",
  });

  const pickupSource = gitText([
    "show",
    `${reconstruction.pickupRevision}:rust/crates/omena-query/src/lib.rs`,
  ]);
  const explicitReexportSource = gitText([
    "show",
    `${reconstruction.explicitReexportRevision}:rust/crates/omena-query/src/lib.rs`,
  ]);
  assert.equal(countNamedPublicReexports(pickupSource), 923);
  assert.equal(countNamedPublicReexports(explicitReexportSource), 923);
  assert.equal(countPublicTypeAliases(pickupSource), 1);
  assert.equal(countPublicTypeAliases(explicitReexportSource), 1);
  assert.equal(reconstruction.pickupEntryGate.namedReexportCount, 923);
  assert.equal(reconstruction.pickupEntryGate.publicTypeAliasCount, 1);
  assert.equal(reconstruction.pickupEntryGate.facadeNameCount, 924);
  assert.equal(
    reconstruction.pickupEntryGate.namedReexportCount +
      reconstruction.pickupEntryGate.publicTypeAliasCount,
    reconstruction.pickupEntryGate.facadeNameCount,
  );

  const pickupDefaultOutput = gitText([
    "show",
    `${reconstruction.pickupRevision}:rust/crates/omena-query/tests/snapshots/public-api.txt`,
  ]);
  const pickupDefaultRootNames = publicApiRootNames(pickupDefaultOutput);
  const currentDefaultRootNames = publicApiRootNames(
    readFileSync(defaultPublicApiSnapshotPath, "utf8"),
  );
  const currentAllFeaturesRootNames = publicApiRootNames(
    readFileSync(allFeaturesPublicApiSnapshotPath, "utf8"),
  );
  assert.equal(pickupDefaultRootNames.size, reconstruction.pickupEntryGate.defaultRootNameCount);
  assert.equal(reconstruction.pickupEntryGate.allFeaturesRootNameCount, 930);
  assert.equal(reconstruction.pickupEntryGate.allFeaturesOnlyRootNameCount, 61);
  assert.equal(
    reconstruction.pickupEntryGate.allFeaturesRootNameCount -
      reconstruction.pickupEntryGate.defaultRootNameCount,
    reconstruction.pickupEntryGate.allFeaturesOnlyRootNameCount,
  );

  const pickupSemverRegister = JSON.parse(
    gitText(["show", `${reconstruction.pickupRevision}:rust/omena-rust-semver-intent.json`]),
  ) as { readonly intents?: readonly unknown[] };
  const currentSemverRegister = JSON.parse(readFileSync(semverIntentPath, "utf8")) as {
    readonly intents?: readonly unknown[];
  };
  assert.equal(pickupSemverRegister.intents?.length ?? 0, 0);
  assert.equal(reconstruction.pickupEntryGate.semverIntentCount, 0);
  assert.equal(currentSemverRegister.intents?.length ?? 0, 6);

  const replay = reconstruction.explicitReexportReplay;
  assert.equal(replay.pickupWildcardReexportCount, 15);
  assert.equal(replay.explicitReexportWildcardReexportCount, 0);
  assert.equal(
    countWildcardReexportsAtRevision(
      reconstruction.pickupRevision,
      reconstruction.expectedChangedPaths,
    ),
    replay.pickupWildcardReexportCount,
  );
  assert.equal(
    countWildcardReexportsAtRevision(
      reconstruction.explicitReexportRevision,
      reconstruction.expectedChangedPaths,
    ),
    replay.explicitReexportWildcardReexportCount,
  );
  validateReplayPlane(replay.defaultFeatures, {
    pickupOutputSha256: "sha256:10f12c59226ecb147d645f3ea5b64ee9b072c962d803086416d7e80cfcea62bb",
    pickupRootNameCount: pickupDefaultRootNames.size,
  });
  validateReplayPlane(replay.allFeatures, {
    pickupOutputSha256: "sha256:f163f27710d6a9c8b5f3c51aac43e37921ac90301b3f9346b44f7de0d5c65abc",
    pickupRootNameCount: 930,
  });

  const accounting = reconstruction.currentSurfaceAccounting;
  assert.equal(currentDefaultRootNames.size, accounting.defaultRootNameCount);
  assert.equal(currentAllFeaturesRootNames.size, accounting.allFeaturesRootNameCount);
  assert.equal(
    currentAllFeaturesRootNames.size - currentDefaultRootNames.size,
    accounting.allFeaturesOnlyRootNameCount,
  );
  assert.equal(
    pickupDefaultRootNames.size - currentDefaultRootNames.size,
    accounting.defaultRootNameLossFromPickup,
  );
  assert.equal(
    reconstruction.pickupEntryGate.allFeaturesRootNameCount - currentAllFeaturesRootNames.size,
    accounting.allFeaturesRootNameLossFromPickup,
  );
  const postPickupPublicAdditionNames = [...currentDefaultRootNames]
    .filter((name) => !pickupDefaultRootNames.has(name))
    .toSorted();
  assert.deepEqual(
    postPickupPublicAdditionNames,
    [...accounting.postPickupPublicAdditionNames].toSorted(),
  );
  assert.equal(postPickupPublicAdditionNames.length, accounting.postPickupPublicAdditionCount);
  for (const name of postPickupPublicAdditionNames) {
    assert.ok(currentAllFeaturesRootNames.has(name), `${name} is not all-features-public`);
  }

  const compiledVisibilityNames = candidate.rows
    .filter((row) => row.outcome === "compiledCrateVisible")
    .map((row) => row.name)
    .toSorted();
  assert.equal(compiledVisibilityNames.length, accounting.compiledVisibilityReductionCount);
  const compiledNamesAbsentFromPickupDefault = compiledVisibilityNames
    .filter((name) => !pickupDefaultRootNames.has(name))
    .toSorted();
  assert.deepEqual(
    compiledNamesAbsentFromPickupDefault,
    [...accounting.compiledVisibilityNamesAbsentFromPickupDefault].toSorted(),
  );
  assert.equal(
    compiledNamesAbsentFromPickupDefault.length,
    accounting.compiledVisibilityNamesAbsentFromPickupDefaultCount,
  );
  for (const name of compiledVisibilityNames) {
    assert.ok(
      !currentAllFeaturesRootNames.has(name),
      `${name} remains on the all-features surface`,
    );
  }

  assert.equal(candidate.defaultFeatureNarrowing.rows.length, accounting.defaultOnlyNarrowingCount);
  for (const row of candidate.defaultFeatureNarrowing.rows) {
    assert.ok(pickupDefaultRootNames.has(row.name), `${row.name} was not public at pickup`);
    assert.ok(!currentDefaultRootNames.has(row.name), `${row.name} remains default-public`);
    assert.ok(currentAllFeaturesRootNames.has(row.name), `${row.name} is not all-features-public`);
  }

  assert.equal(accounting.retiredRefreshSurfaceNames.length, accounting.retiredRefreshSurfaceCount);
  const candidateNames = new Set(candidate.rows.map((row) => row.name));
  for (const name of accounting.retiredRefreshSurfaceNames) {
    assert.ok(pickupDefaultRootNames.has(name), `${name} was not public at pickup`);
    assert.ok(!currentAllFeaturesRootNames.has(name), `${name} remains public after retirement`);
    assert.ok(!candidateNames.has(name), `${name} was incorrectly charged to visibility narrowing`);
  }
  assert.equal(
    candidate.rows.filter((row) => row.outcome === "compatibilityPromiseRestored").length,
    accounting.restoredCompatibilityPromiseCount,
  );
  assert.equal(
    accounting.defaultOnlyNarrowingCount -
      accounting.compiledVisibilityNamesAbsentFromPickupDefaultCount +
      accounting.retiredRefreshSurfaceCount -
      accounting.postPickupPublicAdditionCount,
    accounting.defaultLossResidueCount,
  );
  assert.equal(accounting.defaultLossResidueEquation, "44 - 20 + 3 - 7 = 20");
  assert.equal(
    accounting.defaultRootNameLossFromPickup - accounting.compiledVisibilityReductionCount,
    accounting.defaultLossResidueCount,
  );
}

function validateReplayPlane(
  plane: ReexportReplayPlane,
  pickup: { readonly pickupOutputSha256: string; readonly pickupRootNameCount: number },
): void {
  assert.equal(plane.pickupOutputSha256, pickup.pickupOutputSha256);
  assert.equal(plane.explicitReexportOutputSha256, plane.pickupOutputSha256);
  assert.equal(plane.pickupRootNameCount, pickup.pickupRootNameCount);
  assert.equal(plane.explicitReexportRootNameCount, plane.pickupRootNameCount);
  assert.equal(plane.addedRootNameCount, 0);
  assert.equal(plane.removedRootNameCount, 0);
}

function replayReexportBoundary(candidate: VisibilityExperiment): void {
  const reconstruction = candidate.reexportBoundaryReconstruction;
  const tempRoot = mkdtempSync(path.join(os.tmpdir(), "omena-query-reexport-reconstruction-"));
  const worktreePath = path.join(tempRoot, "worktree");
  const targetPath = path.join(tempRoot, "target");
  let worktreeAdded = false;
  try {
    execFileSync(
      "git",
      ["worktree", "add", "--detach", worktreePath, reconstruction.pickupRevision],
      { cwd: repoRoot, stdio: "ignore" },
    );
    worktreeAdded = true;
    const pickupDefault = runHistoricalPublicApi(worktreePath, targetPath, false);
    const pickupAllFeatures = runHistoricalPublicApi(worktreePath, targetPath, true);
    execFileSync(
      "git",
      ["-C", worktreePath, "checkout", "--detach", reconstruction.explicitReexportRevision],
      { cwd: repoRoot, stdio: "ignore" },
    );
    const explicitDefault = runHistoricalPublicApi(worktreePath, targetPath, false);
    const explicitAllFeatures = runHistoricalPublicApi(worktreePath, targetPath, true);
    assert.equal(
      explicitDefault,
      pickupDefault,
      "default-feature de-glob replay changed public API",
    );
    assert.equal(
      explicitAllFeatures,
      pickupAllFeatures,
      "all-features de-glob replay changed public API",
    );
    assertReplayOutput(
      reconstruction.explicitReexportReplay.defaultFeatures,
      pickupDefault,
      explicitDefault,
    );
    assertReplayOutput(
      reconstruction.explicitReexportReplay.allFeatures,
      pickupAllFeatures,
      explicitAllFeatures,
    );
  } finally {
    if (worktreeAdded) {
      spawnSync("git", ["worktree", "remove", "--force", worktreePath], {
        cwd: repoRoot,
        stdio: "ignore",
      });
    }
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function runHistoricalPublicApi(
  worktreePath: string,
  targetPath: string,
  allFeatures: boolean,
): string {
  return execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      path.join(worktreePath, "rust/Cargo.toml"),
      "-p",
      "omena-query",
      "-sss",
      "--color",
      "never",
      ...(allFeatures ? ["--all-features"] : []),
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, CARGO_TARGET_DIR: targetPath },
      maxBuffer: 64 * 1024 * 1024,
    },
  );
}

function assertReplayOutput(
  plane: ReexportReplayPlane,
  pickupOutput: string,
  explicitReexportOutput: string,
): void {
  assert.equal(sha256Text(pickupOutput), plane.pickupOutputSha256);
  assert.equal(sha256Text(explicitReexportOutput), plane.explicitReexportOutputSha256);
  assert.equal(publicApiRootNames(pickupOutput).size, plane.pickupRootNameCount);
  assert.equal(
    publicApiRootNames(explicitReexportOutput).size,
    plane.explicitReexportRootNameCount,
  );
}

function countNamedPublicReexports(candidateSource: string): number {
  let count = 0;
  const groupedUsePattern = /\bpub\s+use\s+[A-Za-z0-9_:]+::\{([\s\S]*?)\};/gu;
  for (const match of candidateSource.matchAll(groupedUsePattern)) {
    count += (match[1] ?? "")
      .split(",")
      .map((item) => exportedName(item.trim()))
      .filter((name): name is string => name !== null).length;
  }
  const withoutGroupedUses = candidateSource.replace(groupedUsePattern, "");
  const singleUsePattern =
    /\bpub\s+use\s+[A-Za-z0-9_:]+::(?:[A-Za-z0-9_]+)(?:\s+as\s+[A-Za-z0-9_]+)?\s*;/gu;
  count += [...withoutGroupedUses.matchAll(singleUsePattern)].length;
  return count;
}

function countPublicTypeAliases(candidateSource: string): number {
  return [...candidateSource.matchAll(/\bpub\s+type\s+[A-Za-z0-9_]+\b/gu)].length;
}

function countWildcardReexportsAtRevision(
  revision: string,
  sourcePaths: readonly string[],
): number {
  return sourcePaths.reduce((count, sourcePath) => {
    const source = gitText(["show", `${revision}:${sourcePath}`]);
    return (
      count + [...source.matchAll(/^\s*pub\s+use\s+[A-Za-z_][A-Za-z0-9_:]*::\*\s*;/gmu)].length
    );
  }, 0);
}

function publicApiRootNames(publicApi: string): ReadonlySet<string> {
  const names = new Set<string>();
  const pattern =
    /^(?:#\[[^\]]*\]\s*)?pub\s+(?:use|fn|struct|enum|const|type|trait|mod|macro|static|union)\s+omena_query::([A-Za-z_][A-Za-z0-9_]*)(.*)$/gmu;
  for (const match of publicApi.matchAll(pattern)) {
    if (!(match[2] ?? "").startsWith("::")) {
      names.add(match[1]!);
    }
  }
  return names;
}

function sha256Text(value: string): string {
  return `sha256:${createHash("sha256").update(value).digest("hex")}`;
}

function gitText(args: readonly string[]): string {
  return execFileSync("git", [...args], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
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
