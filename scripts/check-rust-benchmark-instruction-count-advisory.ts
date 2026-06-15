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
const toolchainChannel = readToolchainChannel();
const valgrind = runCommand(["valgrind", "--version"]);
const valgrindAvailable = valgrind.exitCode === 0;
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

mkdirSync(path.dirname(artifactPath), { recursive: true });
writeFileSync(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`);

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

if (process.env.CI === "true" && !valgrindAvailable) {
  throw new Error(`valgrind is required in CI for the advisory artifact; artifact=${artifactPath}`);
}

if (benchResult && benchResult.exitCode !== 0) {
  throw new Error(`instruction-count advisory bench failed; artifact=${artifactPath}`);
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
  }),
);

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

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function tailLines(value: string): readonly string[] {
  return value.trim().split(/\r?\n/).filter(Boolean).slice(-20);
}
