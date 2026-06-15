import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

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
  readonly benchmarkFamily: "z5-performance-baseline";
  readonly gateMode: "advisory";
  readonly prBlockingReady: false;
  readonly toolchainChannel: string;
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
  benchmarkFamily: "z5-performance-baseline",
  gateMode: "advisory",
  prBlockingReady: false,
  toolchainChannel,
  valgrindAvailable,
  valgrindVersion: valgrindAvailable ? valgrind.stdout.trim() : null,
  iaiCallgrindVersion: "0.16.1",
  artifactPolicy:
    "scheduled/manual artifact only; missing artifact is a workflow failure; PR blocking is deferred until valgrind compatibility and cost are proven",
  disclosurePolicy:
    "green means recorded, correct, and reproducible; it does not mean fastest",
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
assert.ok(artifact.toolchainChannel.length > 0, "artifact must pin the rust toolchain channel");

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
    toolchainChannel: artifact.toolchainChannel,
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
  valgrindAvailable: boolean,
  compileResult: ReturnType<typeof runCommand>,
  benchResult: ReturnType<typeof runCommand> | null,
): string {
  if (compileResult.exitCode !== 0) {
    return "bench-harness-does-not-compile";
  }
  if (!valgrindAvailable) {
    return "local-valgrind-unavailable-advisory-only";
  }
  if (benchResult?.exitCode === 0) {
    return "valgrind-ran-cleanly-advisory-artifact-recorded";
  }
  return "valgrind-or-iai-run-failed-advisory-only";
}

function readToolchainChannel(): string {
  const toolchainToml = readFileSync("rust-toolchain.toml", "utf8");
  const match = /^channel\s*=\s*"([^"]+)"/m.exec(toolchainToml);
  return match?.[1] ?? "unknown";
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function tailLines(value: string): readonly string[] {
  return value.trim().split(/\r?\n/).filter(Boolean).slice(-20);
}
