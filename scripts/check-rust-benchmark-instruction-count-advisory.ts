import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

type RunnerTool = "iai";

interface BenchmarkMachineSnapshotV0 {
  readonly cpuModel: string;
  readonly cores: number;
  readonly ramBytes: number;
  readonly os: string;
  readonly kernel: string;
  readonly arch: string;
}

interface BenchmarkToolchainSnapshotV0 {
  readonly rustcVersion: string;
  readonly rustcCommitHash: string;
  readonly cargoLockSha256: string;
  readonly nodeVersion: string;
  readonly lightningcssVersion: string;
}

interface BenchmarkCorpusSampleV0 {
  readonly name: string;
  readonly path: string;
  readonly sha256: string;
  readonly byteLength: number;
  readonly lineCount: number;
  readonly dialect: string;
  readonly provenanceSource: string;
}

interface BenchmarkRunnerSnapshotV0 {
  readonly command: readonly string[];
  readonly tool: RunnerTool;
  readonly iterations: number;
  readonly warmup: number;
  readonly measuredOperation: string;
}

interface BenchmarkResultSnapshotV0 {
  readonly lane: string;
  readonly metric: string;
  readonly value: number;
  readonly unit: string;
  readonly varianceOrCI: string;
  readonly runs: number;
}

interface BenchmarkComparisonSnapshotV0 {
  readonly lane: string;
  readonly comparatorLane: string;
  readonly direction: string;
  readonly multiplier: number | null;
  readonly disclosure: string;
}

interface CommandResultSnapshotV0 {
  readonly command: readonly string[];
  readonly exitCode: number | null;
  readonly stdoutSha256: string;
  readonly stderrSha256: string;
  readonly stdoutTail: readonly string[];
  readonly stderrTail: readonly string[];
}

type ReachabilityBitsetDecision = "bitsetInstructionWin" | "noInstructionWin" | "deferred";

interface ReachabilityBitsetResultSnapshotV0 {
  readonly lane: "btreeset" | "bitset";
  readonly benchmarkFunction: string;
  readonly metric: "instructions";
  readonly value: number;
  readonly unit: "Ir";
}

interface ReachabilityBitsetComparisonSnapshotV0 {
  readonly lane: "bitset-vs-btreeset";
  readonly numeratorLane: "bitset";
  readonly denominatorLane: "btreeset";
  readonly multiplier: number | null;
  readonly decision: ReachabilityBitsetDecision;
  readonly disclosure: string;
}

interface ReachabilityBitsetDecisionArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.reachability-bitset-decision";
  readonly generatedAtUtc: string;
  readonly omenaGitSha: string;
  readonly machine: BenchmarkMachineSnapshotV0;
  readonly toolchain: BenchmarkToolchainSnapshotV0 & {
    readonly iaiCallgrindVersion: "0.16.1";
    readonly valgrindVersion: string | null;
  };
  readonly corpus: {
    readonly name: "shared-96-node-reachability-corpus";
    readonly nodeCount: number;
    readonly edgeCount: number;
    readonly source: string;
  };
  readonly runner: {
    readonly command: readonly string[];
    readonly tool: "iai-callgrind";
    readonly measuredOperation: "reachability-closure-representation";
  };
  readonly results: readonly ReachabilityBitsetResultSnapshotV0[];
  readonly comparison: ReachabilityBitsetComparisonSnapshotV0;
  readonly measurementStatus: "recorded" | "runnerUnavailable";
  readonly compileResult: CommandResultSnapshotV0;
  readonly benchResult: CommandResultSnapshotV0 | null;
}

interface InstructionCountAdvisoryArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.instruction-count-advisory";
  readonly generatedAtUtc: string;
  readonly omenaGitSha: string;
  readonly machine: BenchmarkMachineSnapshotV0;
  readonly container: null;
  readonly toolchain: BenchmarkToolchainSnapshotV0;
  readonly corpus: readonly BenchmarkCorpusSampleV0[];
  readonly competitors: readonly [];
  readonly runner: BenchmarkRunnerSnapshotV0;
  readonly results: readonly BenchmarkResultSnapshotV0[];
  readonly comparison: BenchmarkComparisonSnapshotV0;
  readonly vendorReportedFlags: readonly [];
  readonly benchmarkFamily: "z5-performance-baseline";
  readonly gateMode: "advisory";
  readonly prBlockingReady: false;
  readonly valgrindAvailable: boolean;
  readonly valgrindVersion: string | null;
  readonly iaiCallgrindVersion: "0.16.1";
  readonly artifactPolicy: string;
  readonly disclosurePolicy: string;
  readonly feasibilityConclusion: string;
  readonly compileResult: CommandResultSnapshotV0;
  readonly benchResult: CommandResultSnapshotV0 | null;
}

const artifactPath =
  process.env.OMENA_BENCHMARK_ARTIFACT_PATH ??
  path.join("rust", "target", "omena-benchmarks", "z5-instruction-count-advisory.json");
const reachabilityDecisionArtifactPath =
  process.env.OMENA_REACHABILITY_BITSET_ARTIFACT_PATH ??
  path.join("rust", "target", "omena-benchmarks", "reachability-bitset-decision-v0.json");
const toolchainChannel = readToolchainChannel();
const valgrind = runCommand(["valgrind", "--version"]);
const valgrindAvailable = valgrind.exitCode === 0;
if (valgrindAvailable) {
  ensureIaiCallgrindRunner();
}
const rustcVersion = runCommand(["rustc", "--version"]);
const rustcVersionVerbose = runCommand(["rustc", "-vV"]);
const gitSha = runCommand(["git", "rev-parse", "HEAD"]);
const corpus = readStyleCorpus();
const compileCommand = [
  "cargo",
  "bench",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-benchmarks",
  "--bench",
  "z5_instruction_count_advisory",
  "--no-run",
] as const;
const compileResult = runCommand(compileCommand);

const benchCommand = [
  "cargo",
  "bench",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-benchmarks",
  "--bench",
  "z5_instruction_count_advisory",
  "--",
  "--output-format=json",
] as const;
const benchResult = valgrindAvailable ? runCommand(benchCommand) : null;
const reachabilityCompileCommand = [
  "cargo",
  "bench",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-benchmarks",
  "--bench",
  "reachability_bitset_decision",
  "--no-run",
] as const;
const reachabilityCompileResult = runCommand(reachabilityCompileCommand);
const reachabilityBenchCommand = [
  "cargo",
  "bench",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-benchmarks",
  "--bench",
  "reachability_bitset_decision",
  "--",
  "--output-format=json",
  "--save-summary=pretty-json",
  "--separate-targets",
] as const;
const reachabilityBenchResult = valgrindAvailable ? runCommand(reachabilityBenchCommand) : null;
const reachabilityResults =
  reachabilityBenchResult?.exitCode === 0
    ? parseReachabilityBitsetSummaries(reachabilityBenchResult.stdout)
    : [];

const artifact: InstructionCountAdvisoryArtifactV0 = {
  schemaVersion: "0",
  product: "omena-benchmarks.instruction-count-advisory",
  generatedAtUtc: new Date().toISOString(),
  omenaGitSha: gitSha.stdout.trim(),
  machine: readMachineSnapshot(),
  container: null,
  toolchain: {
    rustcVersion: rustcVersion.stdout.trim(),
    rustcCommitHash: parseRustcCommitHash(rustcVersionVerbose.stdout),
    cargoLockSha256: sha256(readFileSync("rust/Cargo.lock", "utf8")),
    nodeVersion: process.version,
    lightningcssVersion: readPackageVersion("lightningcss"),
  },
  corpus,
  competitors: [],
  runner: {
    command: benchCommand,
    tool: "iai",
    iterations: 1,
    warmup: 0,
    measuredOperation: "source-to-product-summary-and-transform-print-identity",
  },
  results: [],
  comparison: {
    lane: "omena",
    comparatorLane: "previous-internal-baseline",
    direction: "not-evaluated-advisory",
    multiplier: null,
    disclosure:
      "No speed claim is made from this artifact; PR blocking is deferred until valgrind compatibility and cost are proven.",
  },
  vendorReportedFlags: [],
  benchmarkFamily: "z5-performance-baseline",
  gateMode: "advisory",
  prBlockingReady: false,
  valgrindAvailable,
  valgrindVersion: valgrindAvailable ? valgrind.stdout.trim() : null,
  iaiCallgrindVersion: "0.16.1",
  artifactPolicy:
    "scheduled/manual artifact only; missing artifact is a workflow failure; PR blocking is deferred until valgrind compatibility and cost are proven",
  disclosurePolicy: "green means recorded, correct, and reproducible; it does not mean fastest",
  feasibilityConclusion: summarizeFeasibility(valgrindAvailable, compileResult, benchResult),
  compileResult: snapshotCommandResult(compileCommand, compileResult),
  benchResult: benchResult ? snapshotCommandResult(benchCommand, benchResult) : null,
};
const reachabilityArtifact: ReachabilityBitsetDecisionArtifactV0 = {
  schemaVersion: "0",
  product: "omena-benchmarks.reachability-bitset-decision",
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
    valgrindVersion: valgrindAvailable ? valgrind.stdout.trim() : null,
  },
  corpus: {
    name: "shared-96-node-reachability-corpus",
    nodeCount: 96,
    edgeCount: 192,
    source: "rust/crates/omena-benchmarks/benches/reachability_bitset_decision.rs",
  },
  runner: {
    command: reachabilityBenchCommand,
    tool: "iai-callgrind",
    measuredOperation: "reachability-closure-representation",
  },
  results: reachabilityResults,
  comparison: buildReachabilityBitsetComparison(reachabilityResults),
  measurementStatus: reachabilityResults.length === 2 ? "recorded" : "runnerUnavailable",
  compileResult: snapshotCommandResult(reachabilityCompileCommand, reachabilityCompileResult),
  benchResult: reachabilityBenchResult
    ? snapshotCommandResult(reachabilityBenchCommand, reachabilityBenchResult)
    : null,
};

mkdirSync(path.dirname(artifactPath), { recursive: true });
writeFileSync(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`);
mkdirSync(path.dirname(reachabilityDecisionArtifactPath), { recursive: true });
writeFileSync(
  reachabilityDecisionArtifactPath,
  `${JSON.stringify(reachabilityArtifact, null, 2)}\n`,
);

assert.equal(artifact.schemaVersion, "0");
assert.equal(artifact.product, "omena-benchmarks.instruction-count-advisory");
assert.equal(artifact.gateMode, "advisory");
assert.equal(artifact.prBlockingReady, false);
assert.equal(toolchainChannel, "1.96.0", "artifact gate must follow the pinned repo toolchain");
assert.ok(artifact.omenaGitSha.length >= 7, "artifact must record the git sha");
assert.ok(artifact.toolchain.rustcVersion.length > 0, "artifact must record rustc");
assert.ok(artifact.toolchain.cargoLockSha256.length > 0, "artifact must hash Cargo.lock");
assert.ok(artifact.corpus.length > 0, "artifact must record the benchmark corpus");

if (compileResult.exitCode !== 0) {
  throw new Error(`instruction-count advisory bench failed to compile; artifact=${artifactPath}`);
}
if (reachabilityCompileResult.exitCode !== 0) {
  throw new Error(
    `reachability bitset decision bench failed to compile; artifact=${reachabilityDecisionArtifactPath}`,
  );
}

if (process.env.CI === "true" && !valgrindAvailable) {
  throw new Error(`valgrind is required in CI for the advisory artifact; artifact=${artifactPath}`);
}

if (benchResult && benchResult.exitCode !== 0) {
  throw new Error(
    `instruction-count advisory bench failed; artifact=${artifactPath}\n${tailLines(benchResult.stderr).join("\n")}`,
  );
}
if (reachabilityBenchResult && reachabilityBenchResult.exitCode !== 0) {
  throw new Error(
    `reachability bitset decision bench failed; artifact=${reachabilityDecisionArtifactPath}\n${tailLines(reachabilityBenchResult.stderr).join("\n")}`,
  );
}
if (valgrindAvailable) {
  assert.equal(
    reachabilityArtifact.measurementStatus,
    "recorded",
    "reachability bitset decision must include instruction counts when valgrind is available",
  );
}

console.log(
  JSON.stringify({
    schemaVersion: artifact.schemaVersion,
    product: artifact.product,
    gateMode: artifact.gateMode,
    prBlockingReady: artifact.prBlockingReady,
    toolchainChannel,
    valgrindAvailable: artifact.valgrindAvailable,
    artifactPath,
    reachabilityDecisionArtifactPath,
    reachabilityDecision: reachabilityArtifact.comparison.decision,
  }),
);

function runCommand(command: readonly string[]) {
  const [executable, ...args] = command;
  const result = spawnSync(executable, args, {
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_HTTP_MULTIPLEXING: process.env.CARGO_HTTP_MULTIPLEXING ?? "false",
      CARGO_TERM_COLOR: "never",
    },
  });
  return {
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function ensureIaiCallgrindRunner() {
  if (runCommand(["iai-callgrind-runner", "--version"]).exitCode === 0) return;
  const command = ["cargo", "install", "iai-callgrind-runner", "--version", "0.16.1", "--locked"];
  let lastInstall: ReturnType<typeof runCommand> | null = null;
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    lastInstall = runCommand(command);
    if (lastInstall.exitCode === 0) return;
  }
  throw new Error(`failed to install iai-callgrind-runner\n${lastInstall?.stderr ?? ""}`);
}

function snapshotCommandResult(
  command: readonly string[],
  result: ReturnType<typeof runCommand>,
): CommandResultSnapshotV0 {
  return {
    command,
    exitCode: result.exitCode,
    stdoutSha256: sha256(result.stdout),
    stderrSha256: sha256(result.stderr),
    stdoutTail: tailLines(result.stdout),
    stderrTail: tailLines(result.stderr),
  };
}

function summarizeFeasibility(
  hasValgrind: boolean,
  compileSnapshot: ReturnType<typeof runCommand>,
  benchSnapshot: ReturnType<typeof runCommand> | null,
): string {
  if (compileSnapshot.exitCode !== 0) {
    return "bench-harness-does-not-compile";
  }
  if (!hasValgrind) {
    return "local-valgrind-unavailable-advisory-only";
  }
  if (benchSnapshot?.exitCode === 0) {
    return "valgrind-ran-cleanly-advisory-artifact-recorded";
  }
  return "valgrind-or-iai-run-failed-advisory-only";
}

function parseReachabilityBitsetSummaries(
  stdout: string,
): readonly ReachabilityBitsetResultSnapshotV0[] {
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
    `iai-callgrind JSON output did not include reachability callgrind summaries; saw keys: ${summaries
      .map((summary) => Object.keys(summary).toSorted().join(","))
      .join(" | ")}`,
  );

  const results = callgrindSummaries
    .map((summary) => {
      const metricSummary = summaryForInstructionMetric(summary);
      const benchmarkFunction = readString(metricSummary, "function_name");
      const lane = reachabilityLaneForBenchmarkFunction(benchmarkFunction);
      if (!lane) return null;
      return {
        lane,
        benchmarkFunction,
        metric: "instructions",
        value: readInstructionCount(metricSummary),
        unit: "Ir",
      } satisfies ReachabilityBitsetResultSnapshotV0;
    })
    .filter((result): result is ReachabilityBitsetResultSnapshotV0 => Boolean(result));
  results.sort((left, right) => left.lane.localeCompare(right.lane));
  assert.deepEqual(results.map((result) => result.lane).toSorted(), ["bitset", "btreeset"]);
  return results;
}

function buildReachabilityBitsetComparison(
  results: readonly ReachabilityBitsetResultSnapshotV0[],
): ReachabilityBitsetComparisonSnapshotV0 {
  const btreeset = results.find((result) => result.lane === "btreeset")?.value;
  const bitset = results.find((result) => result.lane === "bitset")?.value;
  if (!btreeset || !bitset) {
    return {
      lane: "bitset-vs-btreeset",
      numeratorLane: "bitset",
      denominatorLane: "btreeset",
      multiplier: null,
      decision: "deferred",
      disclosure:
        "Instruction-count comparison is deferred until an iai-callgrind runner and valgrind are available.",
    };
  }
  const multiplier = Number((bitset / btreeset).toFixed(6));
  return {
    lane: "bitset-vs-btreeset",
    numeratorLane: "bitset",
    denominatorLane: "btreeset",
    multiplier,
    decision: bitset < btreeset ? "bitsetInstructionWin" : "noInstructionWin",
    disclosure:
      "The decision records measured instruction counts only; it does not create a speed threshold gate.",
  };
}

function reachabilityLaneForBenchmarkFunction(
  functionName: string,
): ReachabilityBitsetResultSnapshotV0["lane"] | null {
  switch (functionName) {
    case "reachability_btreeset_closure_on_shared_corpus":
      return "btreeset";
    case "reachability_bitset_closure_on_shared_corpus":
      return "bitset";
    default:
      return null;
  }
}

function summaryForInstructionMetric(summary: Record<string, unknown>): Record<string, unknown> {
  if (hasObject(summary, "callgrind_summary")) return summary;
  const summaryOutput = readObject(summary, "summary_output");
  const summaryPath = readString(summaryOutput, "path");
  const resolvedSummaryPath = path.isAbsolute(summaryPath)
    ? summaryPath
    : path.resolve(summaryPath);
  return JSON.parse(readFileSync(resolvedSummaryPath, "utf8")) as Record<string, unknown>;
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
  throw new Error("unable to read iai-callgrind v6 Ir instruction count");
}

function readToolchainChannel(): string {
  const toolchainToml = readFileSync("rust-toolchain.toml", "utf8");
  const match = /^channel\s*=\s*"([^"]+)"/m.exec(toolchainToml);
  return match?.[1] ?? "unknown";
}

function readStyleCorpus(): readonly BenchmarkCorpusSampleV0[] {
  const result = runCommand([
    "cargo",
    "run",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-benchmarks",
    "--bin",
    "z5_style_corpus_snapshot",
    "--quiet",
  ]);
  if (result.exitCode !== 0) {
    throw new Error(`unable to read benchmark corpus snapshot:\n${result.stderr}`);
  }
  const snapshot = JSON.parse(result.stdout) as {
    samples: readonly {
      name: string;
      path: string;
      dialect: string;
      byteLength: number;
      lineCount: number;
      source: string;
    }[];
  };
  return snapshot.samples.map((sample) => ({
    name: sample.name,
    path: sample.path,
    sha256: sha256(sample.source),
    byteLength: sample.byteLength,
    lineCount: sample.lineCount,
    dialect: sample.dialect,
    provenanceSource: "rust/crates/omena-benchmarks/src/corpus.rs",
  }));
}

function readMachineSnapshot(): BenchmarkMachineSnapshotV0 {
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
  return value.trim().split(/\r?\n/).filter(Boolean).slice(-20);
}
