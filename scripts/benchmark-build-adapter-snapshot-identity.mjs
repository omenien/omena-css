import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { performance } from "node:perf_hooks";

const require = createRequire(import.meta.url);
const adapter = require("../packages/css-build-adapter/index.cjs");
const binding = require("../rust/crates/omena-napi/pkg");
const hitIterations = numericArgument("--hit-iterations", 150);
const rebuildIterations = numericArgument("--rebuild-iterations", 30);

assert.equal(
  typeof binding.buildSnapshotIdentity,
  "function",
  "build @omena/napi before measuring build snapshot identity overhead",
);

const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-snapshot-benchmark-"));
try {
  const targetPath = path.join(root, "App.module.css");
  const dependencyPath = path.join(root, "tokens.module.css");
  const manifestPath = path.join(root, "package.json");
  const configPath = path.join(root, "omena.config.json");
  const targetSource = Array.from(
    { length: 256 },
    (_, index) => `.item-${index} { color: var(--brand-${index}); }`,
  ).join("\n");
  const dependencySource = `:root {\n${Array.from(
    { length: 256 },
    (_, index) => `  --brand-${index}: rgb(${index % 255} 0 0);`,
  ).join("\n")}\n}`;

  fs.writeFileSync(targetPath, targetSource);
  fs.writeFileSync(dependencyPath, dependencySource);
  fs.writeFileSync(manifestPath, JSON.stringify({ name: "snapshot-benchmark", exports: {} }));
  fs.writeFileSync(configPath, JSON.stringify({ build: { minify: true, sourceMap: false } }));

  const baseOptions = {
    cwd: root,
    engine: binding,
    packageManifests: [manifestPath],
    sources: [dependencyPath],
  };
  const cacheState = adapter.createOmenaBuildState({ cwd: root });
  const warmOptions = await adapter.resolveEffectiveOptions(baseOptions, cacheState);
  const expected = await adapter.rebuildAndCache(targetPath, targetSource, warmOptions, cacheState);
  const hitSamples = [];
  for (let iteration = 0; iteration < hitIterations; iteration += 1) {
    const startedAt = performance.now();
    const options = await adapter.resolveEffectiveOptions(baseOptions, cacheState);
    const output = await adapter.rebuildAndCache(targetPath, targetSource, options, cacheState);
    hitSamples.push(performance.now() - startedAt);
    assert.equal(output.code, expected.code);
  }

  const rebuildMetrics = emptyCacheMetrics();
  const rebuildSamples = [];
  for (let iteration = 0; iteration < rebuildIterations; iteration += 1) {
    const rebuildState = adapter.createOmenaBuildState({ cwd: root });
    const startedAt = performance.now();
    const options = await adapter.resolveEffectiveOptions(baseOptions, rebuildState);
    const output = await adapter.rebuildAndCache(targetPath, targetSource, options, rebuildState);
    rebuildSamples.push(performance.now() - startedAt);
    assert.equal(output.code, expected.code);
    accumulateCacheMetrics(rebuildMetrics, rebuildState.cacheMetrics);
  }

  const hitP50Ms = percentile(hitSamples, 0.5);
  const rebuildP50Ms = percentile(rebuildSamples, 0.5);
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "omena-css-build-adapter.snapshot-identity-benchmark",
        runtime: `${process.platform}-${process.arch} node-${process.versions.node}`,
        measurementShape: {
          moduleInterface: true,
          bundlePassPlanning: "per-enumerated-style-source",
          rebuildDisposition: "fresh-state-digest-miss",
        },
        hitIterations,
        rebuildIterations,
        targetBytes: Buffer.byteLength(targetSource),
        dependencyBytes: Buffer.byteLength(dependencySource),
        manifestBytes: fs.statSync(manifestPath).size,
        configBytes: fs.statSync(configPath).size,
        cacheHitP50Ms: hitP50Ms,
        cacheHitP95Ms: percentile(hitSamples, 0.95),
        forcedRebuildP50Ms: rebuildP50Ms,
        forcedRebuildP95Ms: percentile(rebuildSamples, 0.95),
        rebuildToHitP50Ratio: rebuildP50Ms / hitP50Ms,
        digestEndpointMeanMs:
          Number(cacheState.cacheMetrics.digestComputeNanoseconds) /
          cacheState.cacheMetrics.digestComputations /
          1_000_000,
        forcedRebuildDigestEndpointMeanMs:
          Number(rebuildMetrics.digestComputeNanoseconds) /
          rebuildMetrics.digestComputations /
          1_000_000,
        cacheMetrics: jsonCacheMetrics(cacheState.cacheMetrics),
        forcedRebuildMetrics: jsonCacheMetrics(rebuildMetrics),
        unchangedOutputBytes: Buffer.byteLength(expected.code),
      },
      null,
      2,
    )}\n`,
  );
} finally {
  fs.rmSync(root, { force: true, recursive: true });
}

function numericArgument(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index < 0) return fallback;
  const value = Number(process.argv[index + 1]);
  assert.ok(Number.isInteger(value) && value > 0, `${name} must be a positive integer`);
  return value;
}

function percentile(samples, ratio) {
  const sorted = samples.toSorted((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * ratio))];
}

function jsonCacheMetrics(metrics) {
  return {
    hits: metrics.hits,
    misses: metrics.misses,
    bypasses: metrics.bypasses,
    digestComputations: metrics.digestComputations,
    digestComputeNanoseconds: String(metrics.digestComputeNanoseconds),
    builds: metrics.builds,
    buildNanoseconds: String(metrics.buildNanoseconds),
  };
}

function emptyCacheMetrics() {
  return {
    hits: 0,
    misses: 0,
    bypasses: 0,
    digestComputations: 0,
    digestComputeNanoseconds: 0n,
    builds: 0,
    buildNanoseconds: 0n,
  };
}

function accumulateCacheMetrics(total, current) {
  total.hits += current.hits;
  total.misses += current.misses;
  total.bypasses += current.bypasses;
  total.digestComputations += current.digestComputations;
  total.digestComputeNanoseconds += current.digestComputeNanoseconds;
  total.builds += current.builds;
  total.buildNanoseconds += current.buildNanoseconds;
}
