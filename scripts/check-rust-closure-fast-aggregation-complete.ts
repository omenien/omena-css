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
 * Operability meta-gate. The closure-fast CI job (and any job built the same
 * way) runs each gate as a `continue-on-error: true` step and then aggregates
 * the per-step outcomes in a final `exit "$failed"` loop. If a gate step is
 * added but its `${{ steps.<id>.outcome }}` is NOT wired into that loop, the
 * step can FAIL SILENTLY and CI still goes green. This gate proves, for every
 * job, that every `continue-on-error` step (a) has an id and (b) has its outcome
 * referenced somewhere in the job's aggregation, so a forgotten wiring reds CI
 * instead of hiding a red gate.
 *
 * ci.yml is parsed textually (the repo has no node YAML dependency; this mirrors
 * the string-based workflow assertions in check-rust-m6-publication-material.ts).
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const ciPath = path.join(repoRoot, ".github/workflows/ci.yml");
const lines = readFileSync(ciPath, "utf8").split("\n");

const jobsHeaderIndex = lines.findIndex((line) => /^jobs:\s*$/.test(line));
assert.ok(jobsHeaderIndex >= 0, "ci.yml has no top-level `jobs:` section");

interface JobBlock {
  readonly name: string;
  readonly start: number;
  end: number;
}

// Job headers are 2-space-indented `name:` keys directly under `jobs:`.
const jobs: JobBlock[] = [];
for (let i = jobsHeaderIndex + 1; i < lines.length; i += 1) {
  const line = lines[i];
  if (/^\S/.test(line) && line.trim() !== "") {
    break; // left the jobs: section (a new top-level key)
  }
  const header = line.match(/^ {2}([A-Za-z0-9_-]+):\s*$/);
  if (header) {
    if (jobs.length > 0) {
      jobs[jobs.length - 1].end = i;
    }
    jobs.push({ name: header[1], start: i, end: lines.length });
  }
}
assert.ok(jobs.length > 0, "ci.yml `jobs:` section has no jobs");

const violations: string[] = [];
const summary: Array<{ job: string; gatedSteps: number }> = [];

for (const job of jobs) {
  const block = lines.slice(job.start, job.end);

  // Step boundaries: 6-space-indented `- ` list items under `steps:`.
  const stepStarts: number[] = [];
  for (let i = 0; i < block.length; i += 1) {
    if (/^ {6}- /.test(block[i])) {
      stepStarts.push(i);
    }
  }

  // Collect outcome references that actually GATE the result. A `steps.<id>.outcome`
  // on an `echo` line is debug output only — it does not affect `$failed` — so it
  // must NOT count as aggregated, or a step echoed-but-not-looped would slip through.
  const referencedOutcomes = new Set<string>();
  for (const line of block) {
    if (/^\s*echo\b/.test(line)) {
      continue;
    }
    for (const match of line.matchAll(/steps\.([A-Za-z0-9_-]+)\.outcome/g)) {
      referencedOutcomes.add(match[1]);
    }
  }

  let gatedSteps = 0;
  for (let s = 0; s < stepStarts.length; s += 1) {
    const from = stepStarts[s];
    const to = s + 1 < stepStarts.length ? stepStarts[s + 1] : block.length;
    const stepText = block.slice(from, to).join("\n");
    if (!/continue-on-error:\s*true/.test(stepText)) {
      continue;
    }
    gatedSteps += 1;
    const idMatch = stepText.match(/\bid:\s*([A-Za-z0-9_-]+)/);
    if (!idMatch) {
      violations.push(
        `${job.name}: a continue-on-error step has no id, so its outcome cannot be aggregated`,
      );
      continue;
    }
    const id = idMatch[1];
    if (!referencedOutcomes.has(id)) {
      violations.push(
        `${job.name}: continue-on-error step "${id}" is never referenced as steps.${id}.outcome in the job aggregation (it can fail silently)`,
      );
    }
  }
  if (gatedSteps > 0) {
    summary.push({ job: job.name, gatedSteps });
  }
}

assert.equal(
  violations.length,
  0,
  `closure-fast aggregation is incomplete:\n  ${violations.join("\n  ")}`,
);

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

const ciText = lines.join("\n");
const invokedShards: string[] = [];
for (const match of ciText.matchAll(
  /omena-check run rust\/closure-fast --summary --shard=([A-Za-z0-9_-]+)/g,
)) {
  invokedShards.push(match[1]);
}
assert.deepEqual(
  [...invokedShards].sort(),
  [...expectedShards].sort(),
  `ci.yml must invoke every closure-fast shard exactly once (expected ${expectedShards.join(", ")}; found ${invokedShards.join(", ") || "none"})`,
);
assert.equal(
  /omena-check run rust\/closure-fast --summary(?!\s+--shard=)/.test(ciText),
  false,
  "ci.yml must not run the unsharded closure-fast bundle alongside shards (double execution)",
);

let shardUnionSize = 0;
for (const shardName of expectedShards) {
  shardUnionSize += resolveShardMembers(shardedBundleId, shardName, bundleDeps).size;
}
assert.equal(
  shardUnionSize,
  bundleDeps.length,
  `closure-fast shards must partition the bundle (union ${shardUnionSize} vs deps ${bundleDeps.length})`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.closure-fast-aggregation-complete",
      jobsWithGatedSteps: summary,
      aggregationViolations: 0,
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
