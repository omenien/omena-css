import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

type PerfGateLane = "cold-open-n" | "cold-open-2n" | "memoized-recheck-n" | "memoized-recheck-2n";

interface Z5PerfGateMachineSnapshotV0 {
  readonly cpuModel: string;
  readonly cores: number;
  readonly ramBytes: number;
  readonly os: string;
  readonly kernel: string;
  readonly arch: string;
}

interface Z5PerfGateToolchainSnapshotV0 {
  readonly rustcVersion: string;
  readonly rustcCommitHash: string;
  readonly cargoLockSha256: string;
  readonly nodeVersion: string;
  readonly lightningcssVersion: string;
  readonly iaiCallgrindVersion: "0.16.1";
  readonly valgrindVersion: string;
}

interface Z5PerfGateResultSnapshotV0 {
  readonly lane: PerfGateLane;
  readonly benchmarkFunction: string;
  readonly corpusScale: "N" | "2N";
  readonly metric: "instructions";
  readonly value: number;
  readonly unit: "Ir";
}

interface Z5PerfGateComparisonSnapshotV0 {
  readonly lane: "memoized-recheck-slope" | "cold-open-slope";
  readonly numeratorLane: PerfGateLane;
  readonly denominatorLane: PerfGateLane;
  readonly multiplier: number;
  readonly threshold: number;
  readonly thresholdPolicy: string;
}

interface Z5PerfGateBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.z5-perf-gate-baseline";
  readonly benchmarkFamily: "z5-performance-baseline";
  readonly generatedAtUtc: string;
  readonly omenaGitSha: string;
  readonly machine: Z5PerfGateMachineSnapshotV0;
  readonly toolchain: Z5PerfGateToolchainSnapshotV0;
  readonly runner: {
    readonly command: readonly string[];
    readonly tool: "iai-callgrind";
    readonly measuredOperation: "query-cold-open-and-memoized-recheck";
  };
  readonly results: readonly Z5PerfGateResultSnapshotV0[];
  readonly comparison: readonly Z5PerfGateComparisonSnapshotV0[];
}

const baselinePath = path.join(
  "rust",
  "crates",
  "omena-benchmarks",
  "baselines",
  "z5-perf-gate-baseline-v0.json",
);
const writeMode = process.argv.includes("--write");
const complexitySlopeMode = process.argv.includes("--complexity-slope");
const noRegressionMode = process.argv.includes("--no-regression");
const noRegressionThreshold = 0.03;

if (writeMode) {
  writeBaseline();
} else if (complexitySlopeMode) {
  checkComplexitySlope();
} else if (noRegressionMode) {
  checkNoRegression();
} else {
  checkBaseline();
}

function writeBaseline() {
  const valgrind = runCommand(["valgrind", "--version"]);
  assert.equal(valgrind.exitCode, 0, "writing the z5 perf baseline requires valgrind on PATH");
  ensureIaiCallgrindRunner();

  const benchCommand = z5PerfGateBenchCommand();
  const benchResult = runCommand(benchCommand);
  if (benchResult.exitCode !== 0) {
    throw new Error(`z5 perf gate spine bench failed\n${tailLines(benchResult.stderr).join("\n")}`);
  }

  const results = parseIaiCallgrindSummaries(benchResult.stdout);
  const gitSha = runCommand(["git", "rev-parse", "HEAD"]);
  const rustcVersion = runCommand(["rustc", "--version"]);
  const rustcVersionVerbose = runCommand(["rustc", "-vV"]);
  const baseline: Z5PerfGateBaselineV0 = {
    schemaVersion: "0",
    product: "omena-benchmarks.z5-perf-gate-baseline",
    benchmarkFamily: "z5-performance-baseline",
    generatedAtUtc: new Date().toISOString(),
    omenaGitSha: gitSha.stdout.trim(),
    machine: readMachineSnapshot(),
    toolchain: {
      rustcVersion: rustcVersion.stdout.trim(),
      rustcCommitHash: parseRustcCommitHash(rustcVersionVerbose.stdout),
      cargoLockSha256: sha256(readFileSync("rust/Cargo.lock", "utf8")),
      nodeVersion: process.version,
      lightningcssVersion: readPackageVersion("lightningcss"),
      iaiCallgrindVersion: "0.16.1",
      valgrindVersion: valgrind.stdout.trim(),
    },
    runner: {
      command: benchCommand,
      tool: "iai-callgrind",
      measuredOperation: "query-cold-open-and-memoized-recheck",
    },
    results,
    comparison: buildComparisons(results),
  };
  validateBaseline(baseline);
  mkdirSync(path.dirname(baselinePath), { recursive: true });
  writeFileSync(baselinePath, `${JSON.stringify(baseline, null, 2)}\n`);
  printSummary("updated", baseline);
}

function checkBaseline() {
  const baseline = readBaseline();
  validateBaseline(baseline);
  const compileResult = runCommand([
    "cargo",
    "bench",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-benchmarks",
    "--bench",
    "z5_perf_gate_spine",
    "--no-run",
  ]);
  if (compileResult.exitCode !== 0) {
    throw new Error(
      `z5 perf gate spine bench failed to compile\n${tailLines(compileResult.stderr).join("\n")}`,
    );
  }
  printSummary("checked", baseline);
}

function checkComplexitySlope() {
  const baseline = readBaseline();
  validateBaseline(baseline);
  const currentResults = measureCurrentResults();
  const comparisons = buildComparisons(currentResults);
  for (const comparison of comparisons) {
    assert.ok(
      comparison.multiplier <= comparison.threshold,
      `${comparison.lane} exceeded threshold: ${comparison.multiplier} > ${comparison.threshold}`,
    );
  }
  console.log(
    JSON.stringify({
      schemaVersion: "0",
      product: "omena-benchmarks.z5-perf-complexity-slope",
      baselinePath,
      comparisons,
    }),
  );
}

function checkNoRegression() {
  const baseline = readBaseline();
  validateBaseline(baseline);
  const currentResults = measureCurrentResults();
  const regressions = currentResults
    .map((current) => {
      const baselineResult = resultForLane(baseline.results, current.lane);
      const deltaRatio = (current.value - baselineResult.value) / baselineResult.value;
      return {
        lane: current.lane,
        baseline: baselineResult.value,
        current: current.value,
        deltaRatio: Number(deltaRatio.toFixed(6)),
        threshold: noRegressionThreshold,
      };
    })
    .filter((entry) => entry.deltaRatio > noRegressionThreshold);
  assert.deepEqual(regressions, [], "z5 perf instruction-count regression exceeded threshold");
  console.log(
    JSON.stringify({
      schemaVersion: "0",
      product: "omena-benchmarks.z5-perf-no-regression",
      baselinePath,
      threshold: noRegressionThreshold,
      resultCount: currentResults.length,
    }),
  );
}

function readBaseline(): Z5PerfGateBaselineV0 {
  assert.ok(existsSync(baselinePath), `missing z5 perf baseline: ${baselinePath}`);
  return JSON.parse(readFileSync(baselinePath, "utf8")) as Z5PerfGateBaselineV0;
}

function measureCurrentResults(): readonly Z5PerfGateResultSnapshotV0[] {
  const valgrind = runCommand(["valgrind", "--version"]);
  assert.equal(valgrind.exitCode, 0, "checking z5 perf gates requires valgrind on PATH");
  ensureIaiCallgrindRunner();
  const benchCommand = z5PerfGateBenchCommand();
  const benchResult = runCommand(benchCommand);
  if (benchResult.exitCode !== 0) {
    throw new Error(`z5 perf gate spine bench failed\n${tailLines(benchResult.stderr).join("\n")}`);
  }
  return parseIaiCallgrindSummaries(benchResult.stdout);
}

function parseIaiCallgrindSummaries(stdout: string): readonly Z5PerfGateResultSnapshotV0[] {
  const summaries = stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line) as Record<string, unknown>);
  const callgrindSummaries = summaries.filter(
    (summary) =>
      typeof summary.function_name === "string" &&
      (hasObject(summary, "callgrind_summary") || hasObject(summary, "summary_output")),
  );
  assert.ok(
    callgrindSummaries.length > 0,
    `iai-callgrind JSON output did not include callgrind summaries; saw keys: ${summaries
      .map((summary) => Object.keys(summary).toSorted().join(","))
      .join(" | ")}`,
  );

  const results = callgrindSummaries.map((summary) => {
    const metricSummary = summaryForInstructionMetric(summary);
    const benchmarkFunction = readString(metricSummary, "function_name");
    const lane = laneForBenchmarkFunction(benchmarkFunction);
    const value = readInstructionCount(metricSummary);
    return {
      lane,
      benchmarkFunction,
      corpusScale: lane.endsWith("2n") ? "2N" : "N",
      metric: "instructions",
      value,
      unit: "Ir",
    } satisfies Z5PerfGateResultSnapshotV0;
  });
  results.sort((left, right) => left.lane.localeCompare(right.lane));
  assert.deepEqual(
    results.map((result) => result.lane).toSorted(),
    ["cold-open-2n", "cold-open-n", "memoized-recheck-2n", "memoized-recheck-n"],
    "z5 perf baseline must include cold-open and memoized-recheck lanes at N and 2N",
  );
  return results;
}

function summaryForInstructionMetric(summary: Record<string, unknown>): Record<string, unknown> {
  if (hasObject(summary, "callgrind_summary")) return summary;
  const summaryOutput = readObject(summary, "summary_output");
  const summaryPath = readString(summaryOutput, "path");
  const resolvedSummaryPath = path.isAbsolute(summaryPath)
    ? summaryPath
    : path.resolve(summaryPath);
  const savedSummary = JSON.parse(readFileSync(resolvedSummaryPath, "utf8")) as Record<
    string,
    unknown
  >;
  return savedSummary;
}

function readInstructionCount(summary: Record<string, unknown>): number {
  if (Array.isArray(summary.profiles)) {
    return readV6InstructionCount(summary);
  }
  const callgrindSummary = readObject(summary, "callgrind_summary");
  const callgrindRun = readObject(callgrindSummary, "callgrind_run");
  const total = readObject(callgrindRun, "total");
  const metricSummary = readObject(total, "summary");
  const ir = readObject(metricSummary, "Ir");
  const metrics = readObject(ir, "metrics");
  if ("Left" in metrics) return readNumber(metrics, "Left");
  if ("Both" in metrics) {
    const both = metrics.Both;
    assert.ok(Array.isArray(both), "Ir Both metric must be an array");
    return readArrayNumber(both, 0);
  }
  throw new Error("unable to read Ir instruction count from iai-callgrind summary");
}

function readV6InstructionCount(summary: Record<string, unknown>): number {
  const profiles = summary.profiles;
  assert.ok(Array.isArray(profiles), "expected profiles array in iai-callgrind summary");
  for (const profile of profiles) {
    assert.ok(profile && typeof profile === "object" && !Array.isArray(profile));
    const profileObject = profile as Record<string, unknown>;
    if (!hasObject(profileObject, "summaries")) continue;
    const summaries = readObject(profileObject, "summaries");
    const total = readObject(summaries, "total");
    const toolSummary = readObject(total, "summary");
    if (!hasObject(toolSummary, "Callgrind")) continue;
    const callgrind = readObject(toolSummary, "Callgrind");
    if (!hasObject(callgrind, "Ir")) continue;
    const ir = readObject(callgrind, "Ir");
    const metrics = readObject(ir, "metrics");
    if ("Left" in metrics) return readMetricValue(metrics.Left);
    if ("Both" in metrics) {
      const both = metrics.Both;
      assert.ok(Array.isArray(both), "Ir Both metric must be an array");
      return readMetricValue(both[0]);
    }
  }
  throw new Error("unable to read Ir instruction count from iai-callgrind v6 profiles");
}

function buildComparisons(
  results: readonly Z5PerfGateResultSnapshotV0[],
): readonly Z5PerfGateComparisonSnapshotV0[] {
  return [
    {
      lane: "memoized-recheck-slope",
      numeratorLane: "memoized-recheck-2n",
      denominatorLane: "memoized-recheck-n",
      multiplier: ratio(results, "memoized-recheck-2n", "memoized-recheck-n"),
      threshold: 1.1,
      thresholdPolicy:
        "memoized recheck should remain near-flat across N and 2N because only the edited file fact re-runs",
    },
    {
      lane: "cold-open-slope",
      numeratorLane: "cold-open-2n",
      denominatorLane: "cold-open-n",
      multiplier: ratio(results, "cold-open-2n", "cold-open-n"),
      threshold: 2.2,
      thresholdPolicy:
        "cold open may scale with corpus size; 2.2 leaves deterministic headroom over the ideal 2x slope",
    },
  ];
}

function z5PerfGateBenchCommand(): readonly string[] {
  return [
    "cargo",
    "bench",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-benchmarks",
    "--bench",
    "z5_perf_gate_spine",
    "--",
    "--output-format=json",
    "--save-summary=pretty-json",
    "--separate-targets",
  ];
}

function validateBaseline(baseline: Z5PerfGateBaselineV0) {
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "omena-benchmarks.z5-perf-gate-baseline");
  assert.equal(baseline.benchmarkFamily, "z5-performance-baseline");
  assert.equal(baseline.runner.tool, "iai-callgrind");
  assert.ok(baseline.toolchain.cargoLockSha256.length > 0);
  assert.ok(baseline.toolchain.rustcCommitHash.length > 0);
  assert.ok(baseline.toolchain.lightningcssVersion.length > 0);
  assert.deepEqual(baseline.results.map((result) => result.lane).toSorted(), [
    "cold-open-2n",
    "cold-open-n",
    "memoized-recheck-2n",
    "memoized-recheck-n",
  ]);
  for (const result of baseline.results) {
    assert.equal(result.metric, "instructions");
    assert.equal(result.unit, "Ir");
    assert.ok(Number.isSafeInteger(result.value) && result.value > 0);
  }
  assert.equal(baseline.comparison.length, 2);
}

function ensureIaiCallgrindRunner() {
  if (runCommand(["iai-callgrind-runner", "--version"]).exitCode === 0) return;
  const install = runCommand([
    "cargo",
    "install",
    "iai-callgrind-runner",
    "--version",
    "0.16.1",
    "--locked",
  ]);
  if (install.exitCode !== 0) {
    throw new Error(`failed to install iai-callgrind-runner\n${install.stderr}`);
  }
}

function laneForBenchmarkFunction(functionName: string): PerfGateLane {
  switch (functionName) {
    case "cold_open_query_corpus_n":
      return "cold-open-n";
    case "cold_open_query_corpus_2n":
      return "cold-open-2n";
    case "memoized_recheck_query_corpus_n":
      return "memoized-recheck-n";
    case "memoized_recheck_query_corpus_2n":
      return "memoized-recheck-2n";
    default:
      throw new Error(`unexpected z5 perf benchmark function: ${functionName}`);
  }
}

function ratio(
  results: readonly Z5PerfGateResultSnapshotV0[],
  numeratorLane: PerfGateLane,
  denominatorLane: PerfGateLane,
): number {
  const numerator = results.find((result) => result.lane === numeratorLane)?.value;
  const denominator = results.find((result) => result.lane === denominatorLane)?.value;
  assert.ok(
    numerator && denominator,
    `missing comparison lanes: ${numeratorLane}/${denominatorLane}`,
  );
  return Number((numerator / denominator).toFixed(6));
}

function resultForLane(
  results: readonly Z5PerfGateResultSnapshotV0[],
  lane: PerfGateLane,
): Z5PerfGateResultSnapshotV0 {
  const result = results.find((candidate) => candidate.lane === lane);
  assert.ok(result, `missing z5 perf result lane: ${lane}`);
  return result;
}

function runCommand(command: readonly string[]) {
  const [executable, ...args] = command;
  const result = spawnSync(executable, args, {
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_TERM_COLOR: "never",
    },
  });
  return {
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function readMachineSnapshot(): Z5PerfGateMachineSnapshotV0 {
  return {
    cpuModel: os.cpus()[0]?.model ?? "unknown",
    cores: os.cpus().length,
    ramBytes: os.totalmem(),
    os: os.type(),
    kernel: os.release(),
    arch: os.arch(),
  };
}

function parseRustcCommitHash(output: string): string {
  const match = /^commit-hash:\s*(.+)$/m.exec(output);
  return match?.[1] ?? "unknown";
}

function readPackageVersion(packageName: string): string {
  const packageJson = JSON.parse(
    readFileSync(path.join("node_modules", packageName, "package.json"), "utf8"),
  ) as { version?: string };
  return packageJson.version ?? "unknown";
}

function readObject(object: Record<string, unknown>, key: string): Record<string, unknown> {
  const value = object[key];
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    `expected object at ${key}`,
  );
  return value as Record<string, unknown>;
}

function hasObject(object: Record<string, unknown>, key: string): boolean {
  const value = object[key];
  return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

function readString(object: Record<string, unknown>, key: string): string {
  const value = object[key];
  assert.equal(typeof value, "string", `expected string at ${key}`);
  return value;
}

function readNumber(object: Record<string, unknown>, key: string): number {
  const value = object[key];
  assert.equal(typeof value, "number", `expected number at ${key}`);
  return value;
}

function readArrayNumber(array: readonly unknown[], index: number): number {
  const value = array[index];
  assert.equal(typeof value, "number", `expected number at array index ${index}`);
  return value;
}

function readMetricValue(value: unknown): number {
  if (typeof value === "number") return value;
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "expected iai-callgrind metric object",
  );
  const metric = value as Record<string, unknown>;
  if ("Int" in metric) {
    const intValue = metric.Int;
    assert.equal(typeof intValue, "number", "expected integer metric value");
    return intValue;
  }
  if ("Float" in metric) {
    const floatValue = metric.Float;
    assert.equal(typeof floatValue, "number", "expected float metric value");
    assert.ok(Number.isSafeInteger(floatValue), "instruction count must be an integer metric");
    return floatValue;
  }
  throw new Error("unable to read iai-callgrind metric value");
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function tailLines(value: string): readonly string[] {
  return value.trim().split(/\r?\n/).filter(Boolean).slice(-40);
}

function printSummary(mode: "checked" | "updated", baseline: Z5PerfGateBaselineV0) {
  console.log(
    JSON.stringify({
      schemaVersion: baseline.schemaVersion,
      product: baseline.product,
      mode,
      baselinePath,
      resultCount: baseline.results.length,
      comparisons: baseline.comparison,
    }),
  );
}
