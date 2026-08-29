import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

interface CorpusEntry {
  readonly id: string;
  readonly band: string;
  readonly expectedBytes: number;
  readonly sha256: string;
  readonly localPath: string | null;
  readonly remoteFileName: string | null;
  readonly sourceUrl: string;
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
  readonly normalizedSlopeRatio: number;
  readonly minimumPathP90Nanoseconds: number;
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
  dispositionInputs: {
    minimumAcrossPaths: boolean;
    perBand: boolean;
    absoluteThresholdOnly: boolean;
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
  readonly measurement: "normalized-minimum-path-p90-nanoseconds-per-byte";
  readonly allowedRegressionRatio: number;
  readonly machine: {
    readonly cpuModel: string;
    readonly cores: number;
    readonly os: string;
    readonly arch: string;
  };
  readonly pins: readonly {
    readonly corpusId: string;
    readonly band: string;
    readonly sourceBytes: number;
    normalizedSlopeRatio: number;
    readonly measuredMinimumPathP90Nanoseconds: number;
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
  "minimum-across-paths-per-band-and-two-independent-park-eligible-sources-v0";

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
    report.dispositionInputs.absoluteThresholdOnly = true;
    report.dispositionInputs.minimumAcrossPaths = false;
  }
  validateReport(report, manifest);
  runDispositionMutationSelftest(report, manifest);

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
      baseline.pins[baseline.pins.length - 1]!.normalizedSlopeRatio = 0.000_001;
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
        perBandPinRegression: writeMode ? "not-run-in-writer" : "red",
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
  assert.equal(report.dispositionInputs.minimumAcrossPaths, true);
  assert.equal(report.dispositionInputs.perBand, true);
  assert.equal(report.dispositionInputs.absoluteThresholdOnly, false);
  assert.equal(report.dispositionInputs.requiredIndependentSourceCount, 2);
  assert.match(report.parkScope, /hand-authored, non-minified, non-dist/);
  assert.match(report.yardstickValidation, /unvalidated-wall-clock/);
  assert.match(report.yardstickValidation, /load-bearing-normalized-per-band-pins/);
  assert.equal(report.originalResearchTrigger.realWorldP90Nanoseconds, 4_355_000);
  assert.equal(report.originalResearchTrigger.thresholdNanoseconds, 3_200_000);
  assert.equal(report.originalResearchTrigger.thresholdConsumptionMilli, 1_360);
  assert.equal(report.originalResearchTrigger.fires, true);
  assert.deepEqual(
    report.summaries.map((summary) => summary.band),
    EXPECTED_BANDS,
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
  absoluteOnly.dispositionInputs.minimumAcrossPaths = false;
  absoluteOnly.dispositionInputs.absoluteThresholdOnly = true;
  assert.throws(() => validateReport(absoluteOnly, manifest));
}

function baselineFromReport(report: ParserEditReport, gitSha: string): ParserEditBaseline {
  return {
    schemaVersion: "0",
    product: "omena-benchmarks.parser-edit-slope-baseline",
    generatedAtUtc: new Date().toISOString(),
    omenaGitSha: gitSha,
    measurement: "normalized-minimum-path-p90-nanoseconds-per-byte",
    allowedRegressionRatio: 0.35,
    machine: {
      cpuModel: os.cpus()[0]?.model ?? "unknown",
      cores: os.cpus().length,
      os: `${os.type()} ${os.release()}`,
      arch: os.arch(),
    },
    pins: report.summaries.map((summary) => ({
      corpusId: summary.corpusId,
      band: summary.band,
      sourceBytes: summary.sourceBytes,
      normalizedSlopeRatio: summary.normalizedSlopeRatio,
      measuredMinimumPathP90Nanoseconds: summary.minimumPathP90Nanoseconds,
    })),
  };
}

function validateAgainstBaseline(report: ParserEditReport, baseline: ParserEditBaseline): void {
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "omena-benchmarks.parser-edit-slope-baseline");
  assert.equal(baseline.measurement, "normalized-minimum-path-p90-nanoseconds-per-byte");
  assert.ok(baseline.allowedRegressionRatio > 0 && baseline.allowedRegressionRatio < 1);
  assert.deepEqual(
    baseline.pins.map((pin) => pin.band),
    EXPECTED_BANDS,
    "all five per-band slope pins are load-bearing",
  );
  assert.equal(baseline.pins.length, report.summaries.length);
  for (const [index, summary] of report.summaries.entries()) {
    const pin = baseline.pins[index]!;
    assert.equal(pin.corpusId, summary.corpusId);
    assert.equal(pin.band, summary.band);
    assert.equal(pin.sourceBytes, summary.sourceBytes);
    assert.ok(pin.normalizedSlopeRatio > 0, `${pin.band} must have a positive slope pin`);
    const threshold = pin.normalizedSlopeRatio * (1 + baseline.allowedRegressionRatio);
    assert.ok(
      summary.normalizedSlopeRatio <= threshold,
      `${pin.band} normalized parser-edit slope regressed: current=${summary.normalizedSlopeRatio.toFixed(
        6,
      )} pin=${pin.normalizedSlopeRatio.toFixed(6)} threshold=${threshold.toFixed(6)}`,
    );
  }
}

function runPinMutationSelftest(report: ParserEditReport, baseline: ParserEditBaseline): void {
  const mutation = structuredClone(baseline);
  mutation.pins[mutation.pins.length - 1]!.normalizedSlopeRatio = 0.000_001;
  assert.throws(() => validateAgainstBaseline(report, mutation));
}

function commandOutput(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, { encoding: "utf8" });
  assert.equal(result.status, 0, `${command} ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}
