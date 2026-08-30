import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

interface CorpusEntry {
  readonly id: string;
  readonly band: string;
  readonly sourceProject: string;
  readonly sourceKind: string;
  readonly expectedBytes: number;
  readonly sha256: string;
  readonly localPath: string | null;
  readonly remoteFileName: string | null;
  readonly sourceUrl: string;
  readonly version: string;
  readonly license: string;
  readonly parkEligible: boolean;
  readonly parkScopeReason: string;
}

interface CorpusManifest {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.parser-edit-corpus";
  readonly corpora: readonly CorpusEntry[];
}

interface ParserEditSample {
  readonly comparisonRunIndex: number;
  readonly corpusId: string;
  readonly band: string;
  readonly editShape: string;
  readonly measurementPath: string;
  readonly parseInvocationsPerIteration: number;
  readonly parseTokensPerIteration: number;
}

interface ParserEditSummary {
  readonly corpusId: string;
  readonly band: string;
  readonly sourceBytes: number;
  readonly sourceProject: string;
  readonly sourceKind: string;
  readonly sourceUrl: string;
  readonly sourceVersion: string;
  readonly sourceSha256: string;
  readonly parkEligible: boolean;
  readonly normalizedSlopeRatio: number;
  readonly minimumPathMedianNanoseconds: number;
  readonly minimumPathObservedSpreadNanoseconds: number;
}

interface BandBudgetComparison {
  readonly corpusId: string;
  readonly band: string;
  readonly sourceProject: string;
  readonly sourceKind: string;
  readonly sourceUrl: string;
  readonly sourceVersion: string;
  readonly sourceSha256: string;
  readonly sourceBytes: number;
  readonly parkEligible: boolean;
  readonly pinnedMinimumPathMedianNanoseconds: number;
  readonly observedBaselineSpreadNanoseconds: number;
  readonly allowedMinimumPathMedianNanoseconds: number;
  readonly comparisonRunMinimumPathMedianNanoseconds: readonly number[];
  readonly currentMinimumPathMedianNanoseconds: number;
  readonly withinBudget: boolean;
  readonly reentryThresholdNanoseconds: number;
  readonly reentryCandidate: boolean;
}

interface ParserEditReport {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.parser-edit-trace-slope-profile";
  readonly measurementPin: string;
  readonly releaseBuild: boolean;
  readonly sampleBatchCount: number;
  readonly comparisonRunCount: number;
  readonly corpusManifestSchemaVersion: "0";
  readonly corpusManifestProduct: "omena-benchmarks.parser-edit-corpus";
  readonly editShapes: readonly string[];
  readonly measurementPaths: readonly string[];
  readonly editLocalThresholdNanoseconds: number;
  dispositionPolicy: string;
  readonly parkScope: string;
  readonly yardstickValidation: string;
  readonly originalResearchTrigger: {
    readonly realWorldP90Nanoseconds: number;
    readonly thresholdNanoseconds: number;
    readonly thresholdConsumptionMilli: number;
    readonly fires: boolean;
  };
  readonly baselineProfileId: string;
  readonly executionEnvironment: string;
  readonly comparisonStatistic: string;
  readonly observedSpreadStatistic: string;
  readonly baselineAllowedRegressionRatio: number;
  readonly baselineMeasurementPin: string;
  readonly bandBudgetComparisons: readonly BandBudgetComparison[];
  dispositionInputs: {
    comparisonRunCount: number;
    perBandBudgetCount: number;
    withinBudgetCount: number;
    overBudgetCount: number;
    reentryRegressionRatioMilli: number;
    qualifyingSourceCount: number;
    qualifyingIndependentSourceCount: number;
    requiredIndependentSourceCount: number;
  };
  readonly disposition: string;
  readonly summaries: readonly ParserEditSummary[];
  readonly samples: readonly ParserEditSample[];
}

interface ParserEditBaseline {
  readonly schemaVersion: "1";
  readonly product: "omena-benchmarks.parser-edit-slope-baseline";
  readonly measurement: "absolute-minimum-path-median-nanoseconds";
  readonly profiles: readonly ParserEditBaselineProfile[];
}

interface ParserEditBaselineProfile {
  readonly id: string;
  readonly generatedAtUtc: string;
  readonly omenaGitSha: string;
  readonly executionEnvironment: string;
  readonly statistic: "minimum-across-paths-of-median-across-edit-shape-batch-medians";
  readonly spreadStatistic:
    | "maximum-of-within-run-median-absolute-deviation-and-between-run-range"
    | "maximum-of-observed-edit-shape-median-range-and-between-run-range";
  readonly provenance: {
    readonly kind: "local-writer" | "github-actions-artifact";
    readonly runId: string | null;
  };
  readonly observedMaximumSpreadRatio: number;
  readonly allowedRegressionRatio: number;
  readonly reentryRegressionRatio: 2;
  readonly requiredConsecutiveRunCount: 2;
  readonly requiredIndependentSourceCount: number;
  readonly machine: {
    readonly runnerClass: "local" | "github-hosted";
    readonly cpuModel: string;
    readonly cores: number;
    readonly os: string;
    readonly arch: string;
  };
  readonly pins: readonly {
    readonly corpusId: string;
    readonly band: string;
    readonly sourceProject: string;
    readonly sourceKind: string;
    readonly sourceUrl: string;
    readonly sourceVersion: string;
    readonly sourceSha256: string;
    readonly sourceBytes: number;
    readonly parkEligible: boolean;
    readonly measuredMinimumPathMedianNanoseconds: number;
    readonly observedSpreadNanoseconds: number;
    allowedMinimumPathMedianNanoseconds: number;
  }[];
}

const MANIFEST_PATH = path.join(
  "rust",
  "crates",
  "omena-benchmarks",
  "fixtures",
  "parser-edit-corpus-v0.json",
);
const BASELINE_PATH = path.join(
  "rust",
  "crates",
  "omena-benchmarks",
  "baselines",
  "parser-edit-slope-baseline-v0.json",
);
const DEFAULT_REPORT_PATH = path.join("benchmark-artifacts", "parser-edit-slope-report-v0.json");
const CORPUS_CACHE = path.join("benchmark-artifacts", "parser-edit-corpus");
const EXPECTED_EDIT_SHAPES = [
  "suffix",
  "mid-file-insert",
  "delete",
  "transiently-unbalanced-left-brace",
  "prefix",
] as const;
const EXPECTED_PATHS = [
  "full-parse-and-collection",
  "same-text-reuse-cache-and-cst-facts",
  "edited-text-reuse-cache-and-cst-facts",
] as const;
const EXPECTED_BANDS = ["500B", "8KB", "30KB", "100KB", "1MB"] as const;
const EXPECTED_DISPOSITION_POLICY =
  "absolute-per-band-median-budgets-and-two-consecutive-run-two-independent-source-reentry-v2";
const EXPECTED_STATISTIC =
  "minimum-across-paths-of-median-across-edit-shape-batch-medians" as const;
const EXPECTED_SPREAD_STATISTIC =
  "maximum-of-within-run-median-absolute-deviation-and-between-run-range" as const;
const SUPPORTED_SPREAD_STATISTICS = new Set<ParserEditBaselineProfile["spreadStatistic"]>([
  EXPECTED_SPREAD_STATISTIC,
  "maximum-of-observed-edit-shape-median-range-and-between-run-range",
]);
const PARK_ELIGIBILITY_BY_SOURCE_IDENTITY = new Map<string, boolean>([
  [
    [
      "omena-css",
      "hand-authored",
      "https://github.com/omenien/omena-css/blob/master/examples/src/scenarios/14-non-finite-dynamic/NonFiniteDynamic.module.scss",
      "repository-pin",
      "e9702b58c036ade265d90cbe01efa8688581871115619650ce06eb1d49df8a5e",
    ].join("\0"),
    true,
  ],
  [
    [
      "tailwindcss",
      "hand-authored",
      "https://unpkg.com/tailwindcss@4.3.3/preflight.css",
      "4.3.3",
      "ace8310eed6dc5568a56fc16e1d695cf58da7528d81d66d81649e93cce644df6",
    ].join("\0"),
    true,
  ],
  [
    [
      "tailwindcss",
      "hand-authored",
      "https://unpkg.com/tailwindcss@4.3.3/index.css",
      "4.3.3",
      "175f88737ecb7e033059eac4a3b22a3b5f971d5a05d9fd50811610a0714633b2",
    ].join("\0"),
    true,
  ],
  [
    [
      "fumadocs",
      "distribution",
      "https://unpkg.com/fumadocs-ui@16.15.2/dist/style.css",
      "16.15.2",
      "765821c9236ec61778b7e8b29f662dd9bf9c5c7a5ac667bf0adbf74fd0140d4d",
    ].join("\0"),
    false,
  ],
  [
    [
      "inscada-openbridge-bundle",
      "distribution",
      "https://unpkg.com/@inscada/openbridge-bundle@1.0.2/dist/openbridge.css",
      "1.0.2",
      "b0fa72f204e8d6354c3a534a4a5df8d83d4373421fcbdf2a90db50ad8f3e7a0e",
    ].join("\0"),
    false,
  ],
]);

interface ParserEditExecutionEnvironment {
  readonly profileId: string;
  readonly executionEnvironment: string;
  readonly runnerClass: "local" | "github-hosted";
  readonly cpuModel: string;
  readonly cores: number;
  readonly os: string;
  readonly arch: string;
}

function resolveExecutionEnvironment(): ParserEditExecutionEnvironment {
  const githubHosted = process.env.GITHUB_ACTIONS === "true";
  const detectedOs = os.type();
  const detectedArch = os.arch();
  const profileId =
    process.env.OMENA_PARSER_EDIT_BASELINE_PROFILE ??
    (githubHosted && detectedOs === "Linux" && detectedArch === "x64"
      ? "github-actions-ubuntu-x64"
      : detectedOs === "Darwin" && detectedArch === "arm64"
        ? "local-darwin-arm64"
        : "");
  assert.ok(
    profileId.length > 0,
    `no parser-edit baseline profile for ${detectedOs}/${detectedArch}`,
  );
  const executionEnvironment =
    process.env.OMENA_PARSER_EDIT_EXECUTION_ENVIRONMENT ??
    (profileId === "github-actions-ubuntu-x64"
      ? "github-actions:ubuntu-latest:x64"
      : "local:darwin:arm64");
  return {
    profileId,
    executionEnvironment,
    runnerClass: githubHosted ? "github-hosted" : "local",
    cpuModel: os.cpus()[0]?.model ?? "unknown",
    cores: os.cpus().length,
    os: `${detectedOs} ${os.release()}`,
    arch: detectedArch,
  };
}

function selectedBaselineProfile(
  baseline: ParserEditBaseline,
  profileId: string,
): ParserEditBaselineProfile {
  const profile = baseline.profiles.find((candidate) => candidate.id === profileId);
  assert.ok(profile, `missing parser-edit baseline profile ${profileId}`);
  return profile;
}

function validateExecutionEnvironmentBinding(
  baseline: ParserEditBaseline,
  execution: ParserEditExecutionEnvironment,
): void {
  assert.equal(baseline.schemaVersion, "1");
  assert.equal(baseline.product, "omena-benchmarks.parser-edit-slope-baseline");
  assert.equal(baseline.measurement, "absolute-minimum-path-median-nanoseconds");
  const profile = selectedBaselineProfile(baseline, execution.profileId);
  assert.equal(profile.executionEnvironment, execution.executionEnvironment);
  assert.equal(profile.statistic, EXPECTED_STATISTIC);
  assert.ok(
    SUPPORTED_SPREAD_STATISTICS.has(profile.spreadStatistic),
    `${profile.id} has an unsupported observed-spread statistic`,
  );
  assert.equal(profile.machine.runnerClass, execution.runnerClass);
  assert.equal(profile.machine.arch, execution.arch);
  assert.ok(
    execution.os.startsWith(profile.machine.os.split(" ")[0]!),
    `${profile.id} expected ${profile.machine.os}; observed ${execution.os}`,
  );
  const derivedRatio = Math.ceil(profile.observedMaximumSpreadRatio * 3 * 1_000) / 1_000;
  assert.equal(
    profile.allowedRegressionRatio,
    derivedRatio,
    `${profile.id} budget ratio must be three times its observed spread ratio`,
  );
  const mismatched = structuredClone(baseline);
  (
    selectedBaselineProfile(mismatched, execution.profileId) as {
      executionEnvironment: string;
    }
  ).executionEnvironment += "-mutation";
  assert.throws(
    () => validateExecutionEnvironmentBindingWithoutSelftest(mismatched, execution),
    /Expected values to be strictly equal/u,
  );
}

function validateExecutionEnvironmentBindingWithoutSelftest(
  baseline: ParserEditBaseline,
  execution: ParserEditExecutionEnvironment,
): void {
  const profile = selectedBaselineProfile(baseline, execution.profileId);
  assert.equal(profile.executionEnvironment, execution.executionEnvironment);
  assert.equal(profile.machine.runnerClass, execution.runnerClass);
  assert.equal(profile.machine.arch, execution.arch);
}

export function runParserEditSlopeGate(): void {
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, "utf8")) as CorpusManifest;
  validateManifest(manifest);
  assert.ok(existsSync(BASELINE_PATH), `missing parser-edit baseline: ${BASELINE_PATH}`);
  const baseline = JSON.parse(readFileSync(BASELINE_PATH, "utf8")) as ParserEditBaseline;
  const hostedReportPath = process.argv
    .find((argument) => argument.startsWith("--write-hosted-report="))
    ?.slice("--write-hosted-report=".length);
  if (hostedReportPath !== undefined) {
    writeHostedBaselineFromReport(hostedReportPath, manifest, baseline);
    return;
  }

  materializeCorpora(manifest);
  const gitSha = commandOutput("git", ["rev-parse", "HEAD"]);
  const execution = resolveExecutionEnvironment();
  validateExecutionEnvironmentBinding(baseline, execution);
  const run = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--release",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "parser_edit_trace_slope_profile",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      env: {
        ...process.env,
        OMENA_MEASUREMENT_PIN: gitSha,
        OMENA_PARSER_EDIT_CORPUS_MANIFEST: MANIFEST_PATH,
        OMENA_PARSER_EDIT_CORPUS_ROOT: path.resolve(CORPUS_CACHE),
        OMENA_PARSER_EDIT_BASELINE: path.resolve(BASELINE_PATH),
        OMENA_PARSER_EDIT_BASELINE_PROFILE: execution.profileId,
        OMENA_PARSER_EDIT_EXECUTION_ENVIRONMENT: execution.executionEnvironment,
      },
    },
  );
  assert.equal(
    run.status,
    0,
    `parser-edit slope harness failed\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`,
  );
  const report = JSON.parse(run.stdout) as ParserEditReport;

  if (process.argv.includes("--inject-session-only-disposition")) {
    report.dispositionPolicy = "session-only-p90";
  }
  if (process.argv.includes("--inject-absolute-only-disposition")) {
    report.dispositionPolicy = "absolute-threshold-only";
  }
  validateReport(report, manifest);
  runDispositionMutationSelftest(report, manifest);
  const rustPredicateExecutedTestCount = runRustDispositionPredicateArm();

  const reportPath = process.env.OMENA_PARSER_EDIT_SLOPE_REPORT ?? DEFAULT_REPORT_PATH;
  mkdirSync(path.dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

  const writeMode = process.argv.includes("--write");
  if (writeMode) {
    const updatedBaseline = baselineFromReport(report, gitSha, baseline, execution);
    mkdirSync(path.dirname(BASELINE_PATH), { recursive: true });
    writeFileSync(BASELINE_PATH, `${JSON.stringify(updatedBaseline, null, 2)}\n`);
  } else {
    if (process.argv.includes("--inject-pin-regression")) {
      (
        selectedBaselineProfile(baseline, execution.profileId).pins[0] as {
          allowedMinimumPathMedianNanoseconds: number;
        }
      ).allowedMinimumPathMedianNanoseconds = 0;
    }
    validateAgainstBaseline(report, baseline, execution);
    runPinMutationSelftest(report, baseline, execution);
  }

  console.log(
    JSON.stringify({
      schemaVersion: "0",
      product: "rust.parser-edit-slope-gate",
      corpusCount: report.summaries.length,
      editShapeCount: report.editShapes.length,
      measurementPathCount: report.measurementPaths.length,
      sampleCount: report.samples.length,
      comparisonRunCount: report.comparisonRunCount,
      disposition: report.disposition,
      originalResearchTriggerFires: report.originalResearchTrigger.fires,
      reportPath,
      baselinePath: BASELINE_PATH,
      baselineProfileId: report.baselineProfileId,
      executionEnvironment: report.executionEnvironment,
      writeMode,
      mutationSelftests: {
        sessionOnlyDisposition: "red",
        absoluteThresholdOnlyDisposition: "red",
        everyAbsolutePerBandPin: writeMode ? "not-run-in-writer" : "red",
        rustDispositionPredicate: "red-on-decision-mutation",
        rustDispositionPredicateExecutedTestCount: rustPredicateExecutedTestCount,
      },
    }),
  );
}

function writeHostedBaselineFromReport(
  reportPath: string,
  manifest: CorpusManifest,
  baseline: ParserEditBaseline,
): void {
  const runId = requiredArgument("--hosted-run-id=");
  const generatedAtUtc = requiredArgument("--hosted-generated-at=");
  assert.match(runId, /^\d+$/u);
  assert.match(generatedAtUtc, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/u);
  const report = JSON.parse(readFileSync(reportPath, "utf8")) as ParserEditReport;
  validateReport(report, manifest);
  assert.equal(report.baselineProfileId, "github-actions-ubuntu-x64");
  assert.equal(report.executionEnvironment, "github-actions:ubuntu-latest:x64");
  assert.match(report.measurementPin, /^[0-9a-f]{40}$/u);
  const previous = selectedBaselineProfile(baseline, report.baselineProfileId);
  assert.equal(previous.machine.runnerClass, "github-hosted");
  assert.equal(previous.machine.arch, "x64");

  const pins = report.bandBudgetComparisons.map((comparison, index) => {
    const summary = report.summaries[index]!;
    const measuredMinimumPathMedianNanoseconds = Math.min(
      ...comparison.comparisonRunMinimumPathMedianNanoseconds,
    );
    const betweenRunRange =
      Math.max(...comparison.comparisonRunMinimumPathMedianNanoseconds) -
      measuredMinimumPathMedianNanoseconds;
    return {
      corpusId: summary.corpusId,
      band: summary.band,
      sourceProject: summary.sourceProject,
      sourceKind: summary.sourceKind,
      sourceUrl: summary.sourceUrl,
      sourceVersion: summary.sourceVersion,
      sourceSha256: summary.sourceSha256,
      sourceBytes: summary.sourceBytes,
      parkEligible: summary.parkEligible,
      measuredMinimumPathMedianNanoseconds,
      observedSpreadNanoseconds: Math.max(
        summary.minimumPathObservedSpreadNanoseconds,
        betweenRunRange,
      ),
      allowedMinimumPathMedianNanoseconds: 0,
    };
  });
  const observedMaximumSpreadRatio = Math.max(
    ...pins.map((pin) => pin.observedSpreadNanoseconds / pin.measuredMinimumPathMedianNanoseconds),
  );
  const allowedRegressionRatio = Math.ceil(observedMaximumSpreadRatio * 3 * 1_000) / 1_000;
  assert.ok(allowedRegressionRatio > 0 && allowedRegressionRatio < 1);
  for (const pin of pins) {
    pin.allowedMinimumPathMedianNanoseconds = Math.ceil(
      pin.measuredMinimumPathMedianNanoseconds * (1 + allowedRegressionRatio),
    );
  }
  const profile: ParserEditBaselineProfile = {
    id: report.baselineProfileId,
    generatedAtUtc,
    omenaGitSha: report.measurementPin,
    executionEnvironment: report.executionEnvironment,
    statistic: EXPECTED_STATISTIC,
    spreadStatistic: EXPECTED_SPREAD_STATISTIC,
    provenance: { kind: "github-actions-artifact", runId },
    observedMaximumSpreadRatio,
    allowedRegressionRatio,
    reentryRegressionRatio: 2,
    requiredConsecutiveRunCount: 2,
    requiredIndependentSourceCount: 2,
    machine: {
      ...previous.machine,
      cpuModel: `GitHub-hosted x64 pool recorded by run ${runId}`,
    },
    pins,
  };
  const updated: ParserEditBaseline = {
    ...baseline,
    profiles: [
      ...baseline.profiles.filter((candidate) => candidate.id !== profile.id),
      profile,
    ].toSorted((left, right) => (left.id < right.id ? -1 : left.id > right.id ? 1 : 0)),
  };
  writeFileSync(BASELINE_PATH, `${JSON.stringify(updated, null, 2)}\n`);
  console.log(
    JSON.stringify({
      schemaVersion: "0",
      product: "rust.parser-edit-slope-hosted-baseline-writer",
      reportPath,
      baselinePath: BASELINE_PATH,
      baselineProfileId: profile.id,
      measurementPin: profile.omenaGitSha,
      runId,
      comparisonRunCount: report.comparisonRunCount,
      sampleCount: report.samples.length,
      observedMaximumSpreadRatio,
      allowedRegressionRatio,
    }),
  );
}

function requiredArgument(prefix: string): string {
  const value = process.argv.find((argument) => argument.startsWith(prefix))?.slice(prefix.length);
  assert.ok(value, `missing required ${prefix}<value> argument`);
  return value;
}

function validateManifest(manifest: CorpusManifest): void {
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-benchmarks.parser-edit-corpus");
  assert.deepEqual(
    manifest.corpora.map((entry) => entry.band),
    EXPECTED_BANDS,
    "parser-edit corpus must preserve the five size bands in ascending order",
  );
  assert.equal(new Set(manifest.corpora.map((entry) => entry.id)).size, manifest.corpora.length);
  for (const entry of manifest.corpora) {
    assert.match(entry.sha256, /^[0-9a-f]{64}$/);
    assert.ok(entry.expectedBytes > 0);
    assert.ok(
      (entry.localPath === null) !== (entry.remoteFileName === null),
      `${entry.id} must select exactly one local or remote corpus source`,
    );
    assert.ok(entry.sourceUrl.startsWith("https://"));
    const identity = [
      entry.sourceProject,
      entry.sourceKind,
      entry.sourceUrl,
      entry.version,
      entry.sha256,
    ].join("\0");
    assert.ok(
      PARK_ELIGIBILITY_BY_SOURCE_IDENTITY.has(identity),
      `${entry.id} source identity is not registered for PARK eligibility`,
    );
    assert.equal(
      entry.parkEligible,
      PARK_ELIGIBILITY_BY_SOURCE_IDENTITY.get(identity),
      `${entry.id} PARK eligibility does not match its pinned source identity`,
    );
    assert.ok(entry.parkScopeReason.trim().length > 0);
    assert.ok(entry.license.trim().length > 0);
  }
}

function materializeCorpora(manifest: CorpusManifest): void {
  mkdirSync(CORPUS_CACHE, { recursive: true });
  for (const entry of manifest.corpora) {
    if (entry.localPath !== null) {
      validateCorpusBytes(entry, entry.localPath);
      continue;
    }
    assert.ok(entry.remoteFileName);
    const destination = path.join(CORPUS_CACHE, entry.remoteFileName);
    if (!existsSync(destination) || !corpusBytesMatch(entry, destination)) {
      const temporary = `${destination}.partial`;
      rmSync(temporary, { force: true });
      const download = spawnSync(
        "curl",
        [
          "--location",
          "--fail",
          "--silent",
          "--show-error",
          "--output",
          temporary,
          entry.sourceUrl,
        ],
        { encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
      );
      assert.equal(
        download.status,
        0,
        `failed to fetch pinned parser-edit corpus ${entry.id}: ${download.stderr}`,
      );
      validateCorpusBytes(entry, temporary);
      renameSync(temporary, destination);
    }
    validateCorpusBytes(entry, destination);
  }
}

function corpusBytesMatch(entry: CorpusEntry, filePath: string): boolean {
  try {
    const bytes = readFileSync(filePath);
    return (
      bytes.length === entry.expectedBytes &&
      createHash("sha256").update(bytes).digest("hex") === entry.sha256
    );
  } catch {
    return false;
  }
}

function validateCorpusBytes(entry: CorpusEntry, filePath: string): void {
  const bytes = readFileSync(filePath);
  assert.equal(bytes.length, entry.expectedBytes, `${entry.id} byte length drifted`);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    entry.sha256,
    `${entry.id} content digest drifted`,
  );
}

function validateReport(report: ParserEditReport, manifest: CorpusManifest): void {
  assert.equal(report.schemaVersion, "0");
  assert.equal(report.product, "omena-benchmarks.parser-edit-trace-slope-profile");
  assert.equal(report.releaseBuild, true, "parser-edit slope evidence must use a release build");
  assert.equal(report.comparisonRunCount, 2);
  assert.equal(report.comparisonStatistic, EXPECTED_STATISTIC);
  assert.ok(SUPPORTED_SPREAD_STATISTICS.has(report.observedSpreadStatistic));
  assert.ok(report.baselineAllowedRegressionRatio > 0);
  assert.equal(report.corpusManifestSchemaVersion, "0");
  assert.equal(report.corpusManifestProduct, "omena-benchmarks.parser-edit-corpus");
  assert.deepEqual(report.editShapes, EXPECTED_EDIT_SHAPES);
  assert.deepEqual(report.measurementPaths, EXPECTED_PATHS);
  assert.equal(report.dispositionPolicy, EXPECTED_DISPOSITION_POLICY);
  assert.match(report.baselineMeasurementPin, /^[0-9a-f]{40}$/);
  assert.equal(report.dispositionInputs.comparisonRunCount, 2);
  assert.equal(report.dispositionInputs.perBandBudgetCount, EXPECTED_BANDS.length);
  assert.equal(report.dispositionInputs.reentryRegressionRatioMilli, 2_000);
  assert.equal(report.dispositionInputs.requiredIndependentSourceCount, 2);
  assert.match(report.parkScope, /hand-authored, non-minified, non-dist/);
  assert.match(report.yardstickValidation, /unvalidated-wall-clock/);
  assert.match(report.yardstickValidation, /environment-keyed-absolute-per-band-median-pins/);
  assert.equal(report.originalResearchTrigger.realWorldP90Nanoseconds, 4_355_000);
  assert.equal(report.originalResearchTrigger.thresholdNanoseconds, 3_200_000);
  assert.equal(report.originalResearchTrigger.thresholdConsumptionMilli, 1_360);
  assert.equal(report.originalResearchTrigger.fires, true);
  assert.deepEqual(
    report.summaries.map((summary) => summary.band),
    EXPECTED_BANDS,
  );
  assert.deepEqual(
    report.bandBudgetComparisons.map((comparison) => comparison.band),
    EXPECTED_BANDS,
    "every absolute per-band budget must be represented, including the reference band",
  );
  assert.equal(
    report.dispositionInputs.withinBudgetCount,
    report.bandBudgetComparisons.filter((comparison) => comparison.withinBudget).length,
  );
  assert.equal(
    report.dispositionInputs.overBudgetCount,
    report.bandBudgetComparisons.filter((comparison) => !comparison.withinBudget).length,
  );
  const qualifyingSources = new Set<string>();
  for (const [index, comparison] of report.bandBudgetComparisons.entries()) {
    const summary = report.summaries[index]!;
    assert.equal(comparison.corpusId, summary.corpusId);
    assert.equal(comparison.band, summary.band);
    assert.equal(comparison.sourceProject, summary.sourceProject);
    assert.equal(comparison.sourceKind, summary.sourceKind);
    assert.equal(comparison.sourceUrl, summary.sourceUrl);
    assert.equal(comparison.sourceVersion, summary.sourceVersion);
    assert.equal(comparison.sourceSha256, summary.sourceSha256);
    assert.equal(comparison.sourceBytes, summary.sourceBytes);
    assert.equal(comparison.parkEligible, summary.parkEligible);
    assert.ok(comparison.pinnedMinimumPathMedianNanoseconds > 0);
    assert.ok(comparison.observedBaselineSpreadNanoseconds > 0);
    assert.ok(
      comparison.allowedMinimumPathMedianNanoseconds >
        comparison.pinnedMinimumPathMedianNanoseconds,
    );
    assert.equal(
      comparison.comparisonRunMinimumPathMedianNanoseconds.length,
      report.comparisonRunCount,
    );
    assert.equal(
      comparison.currentMinimumPathMedianNanoseconds,
      Math.min(...comparison.comparisonRunMinimumPathMedianNanoseconds),
    );
    assert.equal(
      comparison.currentMinimumPathMedianNanoseconds,
      summary.minimumPathMedianNanoseconds,
    );
    assert.equal(
      comparison.withinBudget,
      comparison.currentMinimumPathMedianNanoseconds <=
        comparison.allowedMinimumPathMedianNanoseconds,
    );
    assert.equal(
      comparison.reentryThresholdNanoseconds,
      comparison.allowedMinimumPathMedianNanoseconds * 2,
    );
    assert.equal(
      comparison.reentryCandidate,
      comparison.parkEligible &&
        comparison.comparisonRunMinimumPathMedianNanoseconds.every(
          (current) => current >= comparison.reentryThresholdNanoseconds,
        ),
    );
    if (comparison.reentryCandidate) qualifyingSources.add(comparison.sourceProject);
  }
  assert.equal(
    report.dispositionInputs.qualifyingSourceCount,
    report.bandBudgetComparisons.filter((comparison) => comparison.reentryCandidate).length,
  );
  assert.equal(report.dispositionInputs.qualifyingIndependentSourceCount, qualifyingSources.size);
  assert.equal(
    report.disposition,
    qualifyingSources.size >= report.dispositionInputs.requiredIndependentSourceCount
      ? "draft-edit-local-parser-design"
      : "park-edit-local-parser",
    "Rust disposition must be derived from the absolute per-band comparisons",
  );
  assert.equal(
    report.samples.length,
    manifest.corpora.length *
      EXPECTED_EDIT_SHAPES.length *
      EXPECTED_PATHS.length *
      report.comparisonRunCount,
  );
  const matrix = new Set(
    report.samples.map(
      (sample) =>
        `${sample.comparisonRunIndex}#${sample.corpusId}#${sample.editShape}#${sample.measurementPath}`,
    ),
  );
  assert.equal(matrix.size, report.samples.length, "every corpus/shape/path sample must be unique");
  for (
    let comparisonRunIndex = 0;
    comparisonRunIndex < report.comparisonRunCount;
    comparisonRunIndex += 1
  ) {
    for (const entry of manifest.corpora) {
      for (const shape of EXPECTED_EDIT_SHAPES) {
        for (const measurementPath of EXPECTED_PATHS) {
          assert.ok(
            matrix.has(`${comparisonRunIndex}#${entry.id}#${shape}#${measurementPath}`),
            `missing parser-edit sample run=${comparisonRunIndex} ${entry.id}/${shape}/${measurementPath}`,
          );
        }
      }
    }
  }
  for (const sample of report.samples) {
    assert.ok(sample.parseInvocationsPerIteration >= 1);
    assert.ok(sample.parseTokensPerIteration > 0);
  }
}

function runDispositionMutationSelftest(report: ParserEditReport, manifest: CorpusManifest): void {
  const sessionOnly = structuredClone(report);
  sessionOnly.dispositionPolicy = "session-only-p90";
  assert.throws(() => validateReport(sessionOnly, manifest));
  const absoluteOnly = structuredClone(report);
  absoluteOnly.dispositionPolicy = "absolute-threshold-only";
  assert.throws(() => validateReport(absoluteOnly, manifest));
  const fabricatedComparison = structuredClone(report);
  const comparison = fabricatedComparison.bandBudgetComparisons[0];
  assert.ok(comparison);
  (comparison as { reentryCandidate: boolean }).reentryCandidate = !comparison.reentryCandidate;
  assert.throws(
    () => validateReport(fabricatedComparison, manifest),
    /Expected values to be strictly equal/u,
  );
}

function baselineFromReport(
  report: ParserEditReport,
  gitSha: string,
  existing: ParserEditBaseline,
  execution: ParserEditExecutionEnvironment,
): ParserEditBaseline {
  const observedMaximumSpreadRatio = Math.max(
    ...report.summaries.map(
      (summary) =>
        summary.minimumPathObservedSpreadNanoseconds / summary.minimumPathMedianNanoseconds,
    ),
  );
  const allowedRegressionRatio = Math.ceil(observedMaximumSpreadRatio * 3 * 1_000) / 1_000;
  assert.ok(allowedRegressionRatio > 0 && allowedRegressionRatio < 1);
  const profile: ParserEditBaselineProfile = {
    id: execution.profileId,
    generatedAtUtc: new Date().toISOString(),
    omenaGitSha: gitSha,
    executionEnvironment: execution.executionEnvironment,
    statistic: EXPECTED_STATISTIC,
    spreadStatistic: EXPECTED_SPREAD_STATISTIC,
    provenance: { kind: "local-writer", runId: null },
    observedMaximumSpreadRatio,
    allowedRegressionRatio,
    reentryRegressionRatio: 2,
    requiredConsecutiveRunCount: 2,
    requiredIndependentSourceCount: 2,
    machine: {
      runnerClass: execution.runnerClass,
      cpuModel: execution.cpuModel,
      cores: execution.cores,
      os: execution.os,
      arch: execution.arch,
    },
    pins: report.summaries.map((summary) => ({
      corpusId: summary.corpusId,
      band: summary.band,
      sourceProject: summary.sourceProject,
      sourceKind: summary.sourceKind,
      sourceUrl: summary.sourceUrl,
      sourceVersion: summary.sourceVersion,
      sourceSha256: summary.sourceSha256,
      sourceBytes: summary.sourceBytes,
      parkEligible: summary.parkEligible,
      measuredMinimumPathMedianNanoseconds: summary.minimumPathMedianNanoseconds,
      observedSpreadNanoseconds: summary.minimumPathObservedSpreadNanoseconds,
      allowedMinimumPathMedianNanoseconds: Math.ceil(
        summary.minimumPathMedianNanoseconds * (1 + allowedRegressionRatio),
      ),
    })),
  };
  return {
    schemaVersion: "1",
    product: "omena-benchmarks.parser-edit-slope-baseline",
    measurement: "absolute-minimum-path-median-nanoseconds",
    profiles: [
      ...existing.profiles.filter((candidate) => candidate.id !== execution.profileId),
      profile,
    ].toSorted((left, right) => (left.id < right.id ? -1 : left.id > right.id ? 1 : 0)),
  };
}

function validateAgainstBaseline(
  report: ParserEditReport,
  baseline: ParserEditBaseline,
  execution: ParserEditExecutionEnvironment,
): void {
  validateExecutionEnvironmentBindingWithoutSelftest(baseline, execution);
  const profile = selectedBaselineProfile(baseline, execution.profileId);
  assert.equal(report.baselineProfileId, profile.id);
  assert.equal(report.executionEnvironment, profile.executionEnvironment);
  assert.equal(report.comparisonStatistic, profile.statistic);
  assert.equal(report.observedSpreadStatistic, profile.spreadStatistic);
  assert.equal(report.baselineAllowedRegressionRatio, profile.allowedRegressionRatio);
  assert.ok(profile.allowedRegressionRatio > 0 && profile.allowedRegressionRatio < 1);
  assert.equal(profile.reentryRegressionRatio, 2);
  assert.equal(profile.requiredConsecutiveRunCount, 2);
  assert.equal(profile.requiredIndependentSourceCount, 2);
  assert.deepEqual(
    profile.pins.map((pin) => pin.band),
    EXPECTED_BANDS,
    "all five per-band slope pins are load-bearing",
  );
  assert.equal(profile.pins.length, report.summaries.length);
  for (const [index, summary] of report.summaries.entries()) {
    const pin = profile.pins[index]!;
    const comparison = report.bandBudgetComparisons[index]!;
    assert.equal(pin.corpusId, summary.corpusId);
    assert.equal(pin.band, summary.band);
    assert.equal(pin.sourceProject, summary.sourceProject);
    assert.equal(pin.sourceKind, summary.sourceKind);
    assert.equal(pin.sourceUrl, summary.sourceUrl);
    assert.equal(pin.sourceVersion, summary.sourceVersion);
    assert.equal(pin.sourceSha256, summary.sourceSha256);
    assert.equal(pin.sourceBytes, summary.sourceBytes);
    assert.equal(pin.parkEligible, summary.parkEligible);
    assert.ok(
      pin.measuredMinimumPathMedianNanoseconds > 0,
      `${pin.band} must have a positive absolute median pin`,
    );
    const threshold = Math.ceil(
      pin.measuredMinimumPathMedianNanoseconds * (1 + profile.allowedRegressionRatio),
    );
    assert.equal(
      pin.allowedMinimumPathMedianNanoseconds,
      threshold,
      `${pin.band} absolute median budget drifted`,
    );
    assert.equal(
      comparison.pinnedMinimumPathMedianNanoseconds,
      pin.measuredMinimumPathMedianNanoseconds,
    );
    assert.equal(comparison.observedBaselineSpreadNanoseconds, pin.observedSpreadNanoseconds);
    assert.equal(
      comparison.allowedMinimumPathMedianNanoseconds,
      pin.allowedMinimumPathMedianNanoseconds,
    );
    assert.ok(
      summary.minimumPathMedianNanoseconds <= threshold,
      `${pin.band} absolute parser-edit median regressed: current=${summary.minimumPathMedianNanoseconds} pin=${pin.measuredMinimumPathMedianNanoseconds} threshold=${threshold}`,
    );
  }
}

function runPinMutationSelftest(
  report: ParserEditReport,
  baseline: ParserEditBaseline,
  execution: ParserEditExecutionEnvironment,
): void {
  const profile = selectedBaselineProfile(baseline, execution.profileId);
  for (const [index, pin] of profile.pins.entries()) {
    const mutation = structuredClone(baseline);
    (
      selectedBaselineProfile(mutation, execution.profileId).pins[index] as {
        allowedMinimumPathMedianNanoseconds: number;
      }
    ).allowedMinimumPathMedianNanoseconds = 0;
    assert.throws(
      () => validateAgainstBaseline(report, mutation, execution),
      new RegExp(`${pin.band} absolute median budget drifted`, "u"),
    );
  }
}

function runRustDispositionPredicateArm(): number {
  const run = spawnSync(
    "cargo",
    [
      "test",
      "--quiet",
      "--release",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "parser_edit_trace_slope_profile",
      "tests::disposition_mutation_flips_decision_after_two_consecutive_runs",
      "--",
      "--exact",
    ],
    { cwd: process.cwd(), encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  assert.equal(
    run.status,
    0,
    `Rust parser-edit disposition predicate arm failed\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`,
  );
  const output = `${run.stdout ?? ""}${run.stderr ?? ""}`;
  const executed = output.match(/test result: ok\. (\d+) passed;/u);
  assert.ok(executed, `Rust predicate arm did not report an executed test\n${output}`);
  assert.equal(Number(executed[1]), 1, "Rust predicate arm must execute exactly one test");
  assert.match(output, /running 1 test/u);
  return 1;
}

function commandOutput(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, { encoding: "utf8" });
  assert.equal(result.status, 0, `${command} ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}
