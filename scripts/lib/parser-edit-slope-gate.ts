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
  readonly minimumPathP90Nanoseconds: number;
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
  readonly pinnedMinimumPathP90Nanoseconds: number;
  readonly allowedMinimumPathP90Nanoseconds: number;
  readonly currentMinimumPathP90Nanoseconds: number;
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
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.parser-edit-slope-baseline";
  readonly generatedAtUtc: string;
  readonly omenaGitSha: string;
  readonly measurement: "absolute-minimum-path-p90-nanoseconds";
  readonly allowedRegressionRatio: number;
  readonly reentryRegressionRatio: number;
  readonly requiredIndependentSourceCount: number;
  readonly machine: {
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
    readonly measuredMinimumPathP90Nanoseconds: number;
    allowedMinimumPathP90Nanoseconds: number;
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
  "absolute-per-band-p90-budgets-and-two-run-two-independent-source-reentry-v1";
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

export function runParserEditSlopeGate(): void {
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, "utf8")) as CorpusManifest;
  validateManifest(manifest);
  materializeCorpora(manifest);
  const gitSha = commandOutput("git", ["rev-parse", "HEAD"]);
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
  runRustDispositionPredicateArm();

  const reportPath = process.env.OMENA_PARSER_EDIT_SLOPE_REPORT ?? DEFAULT_REPORT_PATH;
  mkdirSync(path.dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

  const writeMode = process.argv.includes("--write");
  if (writeMode) {
    const baseline = baselineFromReport(report, gitSha);
    mkdirSync(path.dirname(BASELINE_PATH), { recursive: true });
    writeFileSync(BASELINE_PATH, `${JSON.stringify(baseline, null, 2)}\n`);
  } else {
    assert.ok(existsSync(BASELINE_PATH), `missing parser-edit baseline: ${BASELINE_PATH}`);
    const baseline = JSON.parse(readFileSync(BASELINE_PATH, "utf8")) as ParserEditBaseline;
    if (process.argv.includes("--inject-pin-regression")) {
      baseline.pins[0]!.allowedMinimumPathP90Nanoseconds = 0;
    }
    validateAgainstBaseline(report, baseline);
    runPinMutationSelftest(report, baseline);
  }

  console.log(
    JSON.stringify({
      schemaVersion: "0",
      product: "rust.parser-edit-slope-gate",
      corpusCount: report.summaries.length,
      editShapeCount: report.editShapes.length,
      measurementPathCount: report.measurementPaths.length,
      sampleCount: report.samples.length,
      disposition: report.disposition,
      originalResearchTriggerFires: report.originalResearchTrigger.fires,
      reportPath,
      baselinePath: BASELINE_PATH,
      writeMode,
      mutationSelftests: {
        sessionOnlyDisposition: "red",
        absoluteThresholdOnlyDisposition: "red",
        everyAbsolutePerBandPin: writeMode ? "not-run-in-writer" : "red",
        rustDispositionPredicate: "green",
      },
    }),
  );
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
  assert.match(report.yardstickValidation, /load-bearing-absolute-per-band-p90-pins/);
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
    assert.ok(comparison.pinnedMinimumPathP90Nanoseconds > 0);
    assert.ok(
      comparison.allowedMinimumPathP90Nanoseconds > comparison.pinnedMinimumPathP90Nanoseconds,
    );
    assert.equal(comparison.currentMinimumPathP90Nanoseconds, summary.minimumPathP90Nanoseconds);
    assert.equal(
      comparison.withinBudget,
      comparison.currentMinimumPathP90Nanoseconds <= comparison.allowedMinimumPathP90Nanoseconds,
    );
    assert.equal(
      comparison.reentryThresholdNanoseconds,
      comparison.pinnedMinimumPathP90Nanoseconds * 2,
    );
    assert.equal(
      comparison.reentryCandidate,
      comparison.parkEligible &&
        comparison.currentMinimumPathP90Nanoseconds >= comparison.reentryThresholdNanoseconds,
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
    manifest.corpora.length * EXPECTED_EDIT_SHAPES.length * EXPECTED_PATHS.length,
  );
  const matrix = new Set(
    report.samples.map(
      (sample) => `${sample.corpusId}#${sample.editShape}#${sample.measurementPath}`,
    ),
  );
  assert.equal(matrix.size, report.samples.length, "every corpus/shape/path sample must be unique");
  for (const entry of manifest.corpora) {
    for (const shape of EXPECTED_EDIT_SHAPES) {
      for (const measurementPath of EXPECTED_PATHS) {
        assert.ok(
          matrix.has(`${entry.id}#${shape}#${measurementPath}`),
          `missing parser-edit sample ${entry.id}/${shape}/${measurementPath}`,
        );
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

function baselineFromReport(report: ParserEditReport, gitSha: string): ParserEditBaseline {
  return {
    schemaVersion: "0",
    product: "omena-benchmarks.parser-edit-slope-baseline",
    generatedAtUtc: new Date().toISOString(),
    omenaGitSha: gitSha,
    measurement: "absolute-minimum-path-p90-nanoseconds",
    allowedRegressionRatio: 0.35,
    reentryRegressionRatio: 2,
    requiredIndependentSourceCount: 2,
    machine: {
      cpuModel: os.cpus()[0]?.model ?? "unknown",
      cores: os.cpus().length,
      os: `${os.type()} ${os.release()}`,
      arch: os.arch(),
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
      measuredMinimumPathP90Nanoseconds: summary.minimumPathP90Nanoseconds,
      allowedMinimumPathP90Nanoseconds: Math.ceil(summary.minimumPathP90Nanoseconds * 1.35),
    })),
  };
}

function validateAgainstBaseline(report: ParserEditReport, baseline: ParserEditBaseline): void {
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "omena-benchmarks.parser-edit-slope-baseline");
  assert.equal(baseline.measurement, "absolute-minimum-path-p90-nanoseconds");
  assert.ok(baseline.allowedRegressionRatio > 0 && baseline.allowedRegressionRatio < 1);
  assert.equal(baseline.reentryRegressionRatio, 2);
  assert.equal(baseline.requiredIndependentSourceCount, 2);
  assert.deepEqual(
    baseline.pins.map((pin) => pin.band),
    EXPECTED_BANDS,
    "all five per-band slope pins are load-bearing",
  );
  assert.equal(baseline.pins.length, report.summaries.length);
  for (const [index, summary] of report.summaries.entries()) {
    const pin = baseline.pins[index]!;
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
      pin.measuredMinimumPathP90Nanoseconds > 0,
      `${pin.band} must have a positive absolute P90 pin`,
    );
    const threshold = Math.ceil(
      pin.measuredMinimumPathP90Nanoseconds * (1 + baseline.allowedRegressionRatio),
    );
    assert.equal(
      pin.allowedMinimumPathP90Nanoseconds,
      threshold,
      `${pin.band} absolute P90 budget drifted`,
    );
    assert.equal(comparison.pinnedMinimumPathP90Nanoseconds, pin.measuredMinimumPathP90Nanoseconds);
    assert.equal(comparison.allowedMinimumPathP90Nanoseconds, pin.allowedMinimumPathP90Nanoseconds);
    assert.ok(
      summary.minimumPathP90Nanoseconds <= threshold,
      `${pin.band} absolute parser-edit P90 regressed: current=${summary.minimumPathP90Nanoseconds} pin=${pin.measuredMinimumPathP90Nanoseconds} threshold=${threshold}`,
    );
  }
}

function runPinMutationSelftest(report: ParserEditReport, baseline: ParserEditBaseline): void {
  for (const [index, pin] of baseline.pins.entries()) {
    const mutation = structuredClone(baseline);
    mutation.pins[index]!.allowedMinimumPathP90Nanoseconds = 0;
    assert.throws(
      () => validateAgainstBaseline(report, mutation),
      new RegExp(`${pin.band} absolute P90 budget drifted`, "u"),
    );
  }
}

function runRustDispositionPredicateArm(): void {
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
      "disposition_requires_two_independent_reentry_sources",
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
}

function commandOutput(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, { encoding: "utf8" });
  assert.equal(result.status, 0, `${command} ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}
