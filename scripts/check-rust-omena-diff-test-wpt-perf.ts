import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

interface WptPerfBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.wpt-case-count-baseline";
  readonly phase: "recorder" | "threshold";
  readonly sourcePin: string;
  readonly caseCount: number;
  readonly thresholdMicrosecondsPerCase: number | null;
  readonly recorderReason: string | null;
  readonly samples: readonly WptPerfSampleV0[];
}

interface WptPerfSampleV0 {
  readonly recordedAtUtc: string;
  readonly gitSha: string;
  readonly worktreeClean: boolean;
  readonly machine: {
    readonly os: string;
    readonly arch: string;
    readonly cpuModel: string;
    readonly cores: number;
  };
  readonly wallTimeMilliseconds: number;
  readonly microsecondsPerCase: number;
}

interface WptRunSummaryV0 {
  readonly extractedSourcePin: string;
  readonly extractedEvaluatedTupleCount: number;
}

interface WptManifestV0 {
  readonly extraction: {
    readonly sourcePin: string;
    readonly tuples: { readonly recordCount: number };
  };
}

const repoRoot = process.cwd();
const baselinePath = path.join(
  repoRoot,
  "rust/crates/omena-benchmarks/baselines/wpt-case-count-baseline-v0.json",
);
const manifest = readJson<WptManifestV0>(
  path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus/manifest.json"),
);
const writeMode = process.argv.includes("--write");
const recordMode = process.argv.includes("--record");
const outputPath = flagValue("--output") ?? process.env.OMENA_WPT_PERF_ARTIFACT_PATH;

if (writeMode) {
  const sample = measureFullCorpus();
  const baseline: WptPerfBaselineV0 = {
    schemaVersion: "0",
    product: "omena-benchmarks.wpt-case-count-baseline",
    phase: "recorder",
    sourcePin: manifest.extraction.sourcePin,
    caseCount: manifest.extraction.tuples.recordCount,
    thresholdMicrosecondsPerCase: null,
    recorderReason:
      "A second comparable scheduled Linux sample is required before enforcing a threshold.",
    samples: [sample],
  };
  writeFileSync(baselinePath, stableJson(baseline));
  process.stdout.write(stableJson(baseline));
} else {
  const baseline = readJson<WptPerfBaselineV0>(baselinePath);
  validateBaseline(baseline);
  verifyThresholdFailurePath();

  if (recordMode) {
    assert.ok(outputPath, "--record requires --output or OMENA_WPT_PERF_ARTIFACT_PATH");
    const sample = measureFullCorpus();
    enforceThreshold(baseline, sample);
    const artifact = {
      schemaVersion: "0",
      product: "omena-benchmarks.wpt-case-count-measurement",
      phase: baseline.phase,
      sourcePin: baseline.sourcePin,
      caseCount: baseline.caseCount,
      thresholdMicrosecondsPerCase: baseline.thresholdMicrosecondsPerCase,
      recorderReason: baseline.recorderReason,
      sample,
    };
    const resolvedOutputPath = path.resolve(repoRoot, outputPath);
    mkdirSync(path.dirname(resolvedOutputPath), { recursive: true });
    writeFileSync(resolvedOutputPath, stableJson(artifact));
    process.stdout.write(stableJson(artifact));
  } else {
    process.stdout.write(
      stableJson({
        product: "omena-benchmarks.wpt-case-count-gate",
        phase: baseline.phase,
        sourcePin: baseline.sourcePin,
        caseCount: baseline.caseCount,
        sampleCount: baseline.samples.length,
        thresholdMicrosecondsPerCase: baseline.thresholdMicrosecondsPerCase,
        recorderReason: baseline.recorderReason,
        thresholdFailurePathVerified: true,
      }),
    );
  }
}

function validateBaseline(baseline: WptPerfBaselineV0): void {
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "omena-benchmarks.wpt-case-count-baseline");
  assert.equal(baseline.sourcePin, manifest.extraction.sourcePin);
  assert.equal(baseline.caseCount, manifest.extraction.tuples.recordCount);
  assert.ok(baseline.caseCount > 0);
  assert.ok(baseline.samples.length > 0);
  for (const sample of baseline.samples) validateSample(sample, baseline.caseCount);
  if (baseline.phase === "recorder") {
    assert.equal(baseline.thresholdMicrosecondsPerCase, null);
    assert.ok(
      baseline.recorderReason?.includes("second comparable scheduled Linux sample"),
      "recorder mode requires a named exit condition",
    );
  } else {
    assert.ok(
      baseline.samples.length >= 2,
      "threshold mode requires two comparable baseline samples",
    );
    assert.ok(
      baseline.thresholdMicrosecondsPerCase !== null && baseline.thresholdMicrosecondsPerCase > 0,
    );
    assert.equal(baseline.recorderReason, null);
  }
}

function measureFullCorpus(): WptPerfSampleV0 {
  const started = process.hrtime.bigint();
  const run = spawnSync(
    process.execPath,
    ["--import", "tsx", "./scripts/check-rust-omena-diff-test-wpt-seed.ts"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, OMENA_WPT_FULL_CORPUS: "1" },
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  const elapsedNanoseconds = process.hrtime.bigint() - started;
  assert.equal(run.status, 0, run.stderr || run.stdout);
  const summary = JSON.parse(run.stdout) as WptRunSummaryV0;
  assert.equal(summary.extractedSourcePin, manifest.extraction.sourcePin);
  assert.equal(summary.extractedEvaluatedTupleCount, manifest.extraction.tuples.recordCount);
  const wallTimeMilliseconds = Number(elapsedNanoseconds) / 1_000_000;
  const sample: WptPerfSampleV0 = {
    recordedAtUtc: new Date().toISOString(),
    gitSha: gitSha(),
    worktreeClean: isWorktreeClean(),
    machine: {
      os: `${os.type()} ${os.release()}`,
      arch: os.arch(),
      cpuModel: os.cpus()[0]?.model ?? "unknown",
      cores: os.cpus().length,
    },
    wallTimeMilliseconds: round(wallTimeMilliseconds),
    microsecondsPerCase: round(
      (wallTimeMilliseconds * 1000) / summary.extractedEvaluatedTupleCount,
    ),
  };
  validateSample(sample, summary.extractedEvaluatedTupleCount);
  return sample;
}

function enforceThreshold(baseline: WptPerfBaselineV0, sample: WptPerfSampleV0): void {
  if (baseline.phase === "recorder") return;
  assert.ok(baseline.thresholdMicrosecondsPerCase !== null);
  assert.ok(
    sample.microsecondsPerCase <= baseline.thresholdMicrosecondsPerCase,
    `WPT case-count runtime ${sample.microsecondsPerCase}us/case exceeds ${baseline.thresholdMicrosecondsPerCase}us/case`,
  );
}

function verifyThresholdFailurePath(): void {
  const fixtureBaseline: WptPerfBaselineV0 = {
    schemaVersion: "0",
    product: "omena-benchmarks.wpt-case-count-baseline",
    phase: "threshold",
    sourcePin: manifest.extraction.sourcePin,
    caseCount: manifest.extraction.tuples.recordCount,
    thresholdMicrosecondsPerCase: 100,
    recorderReason: null,
    samples: [syntheticSample(90), syntheticSample(95)],
  };
  assert.throws(
    () => enforceThreshold(fixtureBaseline, syntheticSample(101)),
    /exceeds 100us\/case/u,
  );
}

function syntheticSample(microsecondsPerCase: number): WptPerfSampleV0 {
  return {
    recordedAtUtc: "2026-01-01T00:00:00.000Z",
    gitSha: "0".repeat(40),
    worktreeClean: true,
    machine: { os: "fixture", arch: "fixture", cpuModel: "fixture", cores: 1 },
    wallTimeMilliseconds: round(
      (microsecondsPerCase * manifest.extraction.tuples.recordCount) / 1000,
    ),
    microsecondsPerCase,
  };
}

function validateSample(sample: WptPerfSampleV0, caseCount: number): void {
  assert.match(sample.recordedAtUtc, /^\d{4}-\d{2}-\d{2}T/u);
  assert.match(sample.gitSha, /^[0-9a-f]{40}$/u);
  assert.equal(typeof sample.worktreeClean, "boolean");
  assert.ok(sample.machine.os.length > 0);
  assert.ok(sample.machine.arch.length > 0);
  assert.ok(sample.machine.cpuModel.length > 0);
  assert.ok(sample.machine.cores > 0);
  assert.ok(sample.wallTimeMilliseconds > 0);
  assert.ok(sample.microsecondsPerCase > 0);
  assert.ok(
    Math.abs(sample.wallTimeMilliseconds * 1000 - sample.microsecondsPerCase * caseCount) <
      caseCount,
    "wall-time and per-case metrics disagree",
  );
}

function gitSha(): string {
  const run = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(run.status, 0, run.stderr);
  return run.stdout.trim();
}

function isWorktreeClean(): boolean {
  const run = spawnSync("git", ["status", "--porcelain", "--untracked-files=no"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(run.status, 0, run.stderr);
  return run.stdout.trim().length === 0;
}

function flagValue(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  assert.ok(value && !value.startsWith("--"), `${name} requires a value`);
  return value;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function stableJson(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function round(value: number): number {
  return Math.round(value * 1000) / 1000;
}
