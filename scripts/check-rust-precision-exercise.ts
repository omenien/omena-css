import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  assertPrecisionFlip,
  precisionExerciseInventory,
  readPrecisionVector,
  sha256,
  type PrecisionExerciseAuthority,
} from "./lib/precision-exercise";
import { type PrecisionExerciseCase } from "./lib/precision-exercise-cases";
import {
  RUST_PRODUCT_TEST_SHARDS,
  rustProductTestCargoInvocations,
} from "./lib/rust-product-test-plan";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authority = JSON.parse(
  readFileSync(path.join(root, "rust/omena-precision-floor-authority.json"), "utf8"),
) as PrecisionExerciseAuthority;
const inventory = precisionExerciseInventory(root, authority);
const requested = process.argv.slice(2);
assert.ok(
  requested.length === 1 &&
    ["--all", "--inventory", ...inventory.pairs.map(({ id }) => id)].includes(requested[0]!),
  "usage: check-rust-precision-exercise.ts <probe-id|--all|--inventory>",
);

const plan = RUST_PRODUCT_TEST_SHARDS.flatMap(rustProductTestCargoInvocations);
const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
    { cwd: root, encoding: "utf8" },
  ),
) as {
  readonly packages: readonly { readonly name: string; readonly id: string }[];
  readonly workspace_members: readonly string[];
  readonly target_directory: string;
};
const members = new Set(
  metadata.packages
    .filter(({ id }) => metadata.workspace_members.includes(id))
    .map(({ name }) => name),
);
const membership = inventory.pairs.map((pair) => {
  assert.ok(members.has(pair.owningCrate), `fixture crate is not a workspace member ${pair.id}`);
  const invocation = plan.find(({ args }) => {
    const selected = args.flatMap((value, index) => (value === "-p" ? [args[index + 1]!] : []));
    const excluded = args.flatMap((value, index) =>
      value === "--exclude" ? [args[index + 1]!] : [],
    );
    return (
      args[0] === "test" &&
      args.includes("--all-features") &&
      ((args.includes("--workspace") && !excluded.includes(pair.owningCrate)) ||
        selected.includes(pair.owningCrate))
    );
  });
  assert.ok(invocation, `probe census-only ${pair.id}`);
  return {
    id: pair.id,
    fixtureSha256: pair.fixtureSha256,
    productGate: "rust/product-test-execution",
    invocation,
  };
});
const computeCondition = {
  platform: process.platform,
  architecture: process.arch,
  cargoVersion: execFileSync("cargo", ["--version"], { cwd: root, encoding: "utf8" }).trim(),
  targetDirectory: metadata.target_directory,
  targetPresentAtStart: existsSync(metadata.target_directory),
  rustcWrapper: process.env.RUSTC_WRAPPER ?? null,
  cargoIncremental: process.env.CARGO_INCREMENTAL ?? null,
  cargoBuildJobs: process.env.CARGO_BUILD_JOBS ?? null,
  omenaSccache: process.env.OMENA_SCCACHE ?? null,
};
process.stdout.write(
  `${JSON.stringify({ product: "rust.precision-exercise", n: inventory.pairs.length, perCrate: inventory.perCrate, noProbe: inventory.noProbe, censusOnly: inventory.censusOnly, countedRows: inventory.countedRows, membership, computeCondition })}\n`,
);
if (requested[0] === "--inventory") process.exit(0);

const started = performance.now();
const selected =
  requested[0] === "--all"
    ? inventory.pairs
    : inventory.pairs.filter(({ id }) => id === requested[0]);
for (const pair of selected) runPair(pair);
process.stdout.write(
  `${JSON.stringify({ product: "rust.precision-exercise-summary", n: inventory.pairs.length, executed: selected.length, durationMs: Math.round(performance.now() - started), perCrate: inventory.perCrate })}\n`,
);

function execute(pair: PrecisionExerciseCase, phase: string) {
  const command = [
    "cargo",
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    pair.owningCrate,
    "--all-features",
    pair.testPath,
    "--",
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ];
  const start = performance.now();
  const result = spawnSync(command[0]!, command.slice(1), {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 30 * 60 * 1000,
  });
  const receipt = {
    probeId: pair.id,
    phase,
    command,
    cwd: root,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    durationMs: Math.round(performance.now() - start),
    stdout: result.stdout,
    stderr: result.stderr,
  };
  // Persist the actual subprocess payload in the gate log before interpreting it.
  process.stdout.write(`${JSON.stringify(receipt)}\n`);
  assert.ok(
    !result.error && !result.signal,
    `product execution unavailable ${pair.id}: ${result.error?.message ?? result.signal}`,
  );
  const transcript = `${result.stdout}\n${result.stderr}`;
  return { status: result.status, transcript, vector: readPrecisionVector(transcript, pair) };
}

function runPair(pair: PrecisionExerciseCase): void {
  const before = execute(pair, "baseline");
  assert.equal(before.status, 0, `product baseline failed ${pair.id}`);
  assert.deepEqual(
    before.vector.actualAxes,
    before.vector.expectedAxes,
    `baseline precision mismatch ${pair.id}`,
  );
  const file = path.join(root, pair.mutation.sourcePath);
  const original = readFileSync(file, "utf8");
  assert.equal(
    original.split(pair.mutation.from).length - 1,
    1,
    `probe span not resolvable ${pair.id}`,
  );
  const mutated = original.replace(pair.mutation.from, pair.mutation.to);
  assert.notEqual(mutated, original, `probe does not mutate source ${pair.id}`);
  const patch = unifiedPatch(pair.mutation.sourcePath, original, mutated);
  let changed: readonly string[] = [];
  execFileSync("git", ["apply", "--whitespace=nowarn", "-"], {
    cwd: root,
    input: patch,
    encoding: "utf8",
  });
  try {
    const after = execute(pair, "mutation");
    changed = assertPrecisionFlip(
      pair,
      before.vector,
      after.vector,
      after.status,
      after.transcript,
    );
  } finally {
    execFileSync("git", ["apply", "--reverse", "--whitespace=nowarn", "-"], {
      cwd: root,
      input: patch,
      encoding: "utf8",
    });
    assert.equal(
      readFileSync(file, "utf8"),
      original,
      `precision mutation restore failed ${pair.id}`,
    );
  }
  const restored = execute(pair, "restored");
  assert.equal(restored.status, 0, `restored product test failed ${pair.id}`);
  assert.deepEqual(
    restored.vector.actualAxes,
    before.vector.actualAxes,
    `restored axes changed ${pair.id}`,
  );
  process.stdout.write(
    `${JSON.stringify({ probeId: pair.id, changedAxes: changed, sourceSha256: sha256(original), mutatedSourceSha256: sha256(mutated), patch, restored: true })}\n`,
  );
}

function unifiedPatch(relativePath: string, before: string, after: string): string {
  const a = before.split("\n");
  const b = after.split("\n");
  let prefix = 0;
  while (prefix < a.length && prefix < b.length && a[prefix] === b[prefix]) prefix++;
  let suffix = 0;
  while (
    suffix < a.length - prefix &&
    suffix < b.length - prefix &&
    a[a.length - 1 - suffix] === b[b.length - 1 - suffix]
  )
    suffix++;
  const start = Math.max(0, prefix - 3);
  const aEnd = Math.min(a.length, a.length - suffix + 3);
  const bEnd = Math.min(b.length, b.length - suffix + 3);
  return (
    [
      `diff --git a/${relativePath} b/${relativePath}`,
      `--- a/${relativePath}`,
      `+++ b/${relativePath}`,
      `@@ -${start + 1},${aEnd - start} +${start + 1},${bEnd - start} @@`,
      ...a.slice(start, prefix).map((line) => ` ${line}`),
      ...a.slice(prefix, a.length - suffix).map((line) => `-${line}`),
      ...b.slice(prefix, b.length - suffix).map((line) => `+${line}`),
      ...a.slice(a.length - suffix, aEnd).map((line) => ` ${line}`),
    ].join("\n") + "\n"
  );
}
