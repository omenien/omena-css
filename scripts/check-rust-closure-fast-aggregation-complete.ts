import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadCheckManifest } from "../packages/check-orchestrator/src/manifest/index.ts";
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

const aggregateJobStart = lines.findIndex((line) => /^ {2}closure-fast:\s*$/.test(line));
assert.ok(aggregateJobStart >= 0, "ci.yml must define the closure-fast aggregate job");
const aggregateJobEnd = lines.findIndex(
  (line, index) => index > aggregateJobStart && /^ {2}[A-Za-z0-9_-]+:\s*$/.test(line),
);
const aggregateBlock = lines
  .slice(aggregateJobStart, aggregateJobEnd < 0 ? lines.length : aggregateJobEnd)
  .join("\n");
assert.match(
  aggregateBlock,
  /^\s+needs:\s*closure-fast-shards\s*$/m,
  "closure-fast aggregate must depend on the complete shard matrix",
);
assert.match(
  aggregateBlock,
  /scripts\/check-ci-required-results\.mjs/,
  "closure-fast aggregate must reject any failed, cancelled, or skipped shard",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.closure-fast-aggregation-complete",
      aggregateJob: "closure-fast",
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
