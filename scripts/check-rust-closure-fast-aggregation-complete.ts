import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadCheckManifest } from "../packages/check-orchestrator/src/manifest/index.ts";
import { loadCiWorkflowRegistry } from "../packages/check-orchestrator/src/manifest/ci-workflow.ts";
import {
  bundleShardNames,
  resolveShardMembers,
} from "../packages/check-orchestrator/src/manifest/shards.ts";

/**
 * rust/closure-fast-aggregation-complete
 *
 * Operability meta-gate. The closure-fast bundle is partitioned across a matrix
 * so one slow boundary cannot serialize every other contract. This check keeps
 * the matrix, shard table, bundle membership, and aggregate result job in
 * lockstep.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const ciPath = path.join(repoRoot, ".github/workflows/ci.yml");
const lines = readFileSync(ciPath, "utf8").split("\n");

const jobsHeaderIndex = lines.findIndex((line) => /^jobs:\s*$/.test(line));
assert.ok(jobsHeaderIndex >= 0, "ci.yml has no top-level `jobs:` section");

// Shard coverage: the closure-fast bundle runs sharded across parallel CI jobs.
// Every shard (named shards + the complement "rest") must be invoked EXACTLY ONCE
// in ci.yml, and the shard tables must PARTITION the bundle deps. A deleted shard
// job, a duplicated shard invocation, or a shard pinning a gate that left the
// bundle all red here — no bundle member can silently stop running in CI.
const manifest = loadCheckManifest();
const shardedBundleId = "rust/closure-fast";
const bundleGate = manifest.gates.find((gate) => gate.id === shardedBundleId);
assert.ok(bundleGate, `bundle "${shardedBundleId}" must exist in the check manifest`);
const bundleDeps = (bundleGate.referencedTargetSpecs ?? []).map((spec) => spec.target);
assert.ok(bundleDeps.length > 0, `bundle "${shardedBundleId}" must have members`);

const expectedShards = bundleShardNames(shardedBundleId);
assert.ok(expectedShards.length > 0, `bundle "${shardedBundleId}" must declare shards`);

const preflightJobStart = lines.findIndex((line) => /^ {2}preflight:\s*$/.test(line));
assert.ok(preflightJobStart >= 0, "ci.yml must define the preflight job");
const preflightJobEnd = lines.findIndex(
  (line, index) => index > preflightJobStart && /^ {2}[A-Za-z0-9_-]+:\s*$/.test(line),
);
const preflightBlock = lines
  .slice(preflightJobStart, preflightJobEnd < 0 ? lines.length : preflightJobEnd)
  .join("\n");
assert.match(
  preflightBlock,
  /^\s+closure-fast-shards:\s*\$\{\{\s*steps\.closure-fast-shards\.outputs\.matrix\s*\}\}\s*$/m,
  "preflight must expose the generated shard matrix as a job output",
);
assert.match(
  preflightBlock,
  /omena-check shards rust\/closure-fast --json/,
  "preflight must derive closure-fast shards from the check manifest",
);

const matrixJobStart = lines.findIndex((line) => /^ {2}closure-fast-shards:\s*$/.test(line));
assert.ok(matrixJobStart >= 0, "ci.yml must define the closure-fast-shards matrix job");
const matrixJobEnd = lines.findIndex(
  (line, index) => index > matrixJobStart && /^ {2}[A-Za-z0-9_-]+:\s*$/.test(line),
);
const matrixBlock = lines.slice(matrixJobStart, matrixJobEnd < 0 ? lines.length : matrixJobEnd);
const matrixText = matrixBlock.join("\n");
assert.match(
  matrixText,
  /^\s+needs:\s*preflight\s*$/m,
  "closure-fast-shards must depend on the matrix-producing preflight job",
);
assert.match(
  matrixText,
  /^\s+shard:\s*\$\{\{\s*fromJSON\(needs\.preflight\.outputs\.closure-fast-shards\)\s*\}\}\s*$/m,
  "closure-fast-shards must consume the generated matrix without a second shard list",
);
assert.match(
  matrixText,
  /^\s+CLOSURE_FAST_SHARD:\s*\$\{\{\s*matrix\.shard\s*\}\}\s*$/m,
  "closure-fast-shards must bind the matrix value outside the shell command",
);
assert.match(
  matrixText,
  /omena-check run rust\/closure-fast --summary --shard="\$CLOSURE_FAST_SHARD"/,
  "closure-fast-shards must execute the bound matrix shard exactly once",
);
const invokedShards = [...expectedShards];

let shardUnionSize = 0;
for (const shardName of expectedShards) {
  shardUnionSize += resolveShardMembers(shardedBundleId, shardName, bundleDeps).size;
}
assert.equal(
  shardUnionSize,
  bundleDeps.length,
  `closure-fast shards must partition the bundle (union ${shardUnionSize} vs deps ${bundleDeps.length})`,
);

// g131-S3 (registry-anchored aggregation): the intermediate closure-fast
// aggregate job was legally folded away — the invariant is that the shard
// matrix's RESULT reaches ci-required through the needs graph (any
// aggregation shape or none; GitHub's skip-cascade makes every needs
// ancestor failure-propagating, and the orchestrator's judge rules govern
// the root aggregator's if:/judge semantics).
const registry = loadCiWorkflowRegistry(repoRoot);
assert.ok(registry, "packages/check-orchestrator/ci-workflow.json must exist");
const needsByName = new Map(registry.jobs.map((job) => [job.name, job.needs]));
const requiredClosure = new Set<string>();
const closureQueue = [...(needsByName.get("ci-required") ?? [])];
while (closureQueue.length > 0) {
  const name = closureQueue.pop();
  if (!name || requiredClosure.has(name)) continue;
  requiredClosure.add(name);
  closureQueue.push(...(needsByName.get(name) ?? []));
}
assert.ok(
  requiredClosure.has("closure-fast-shards"),
  "closure-fast-shards must reach ci-required through the needs graph; " +
    "a failed shard could stop blocking the merge",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.closure-fast-aggregation-complete",
      aggregation: "ci-required needs closure (registry-anchored)",
      matrixJob: "closure-fast-shards",
      shardCoverage: {
        bundle: shardedBundleId,
        shards: expectedShards,
        invoked: invokedShards,
        memberCount: bundleDeps.length,
        partitioned: true,
      },
    },
    null,
    2,
  )}\n`,
);
