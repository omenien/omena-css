import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { transform as lightningTransform } from "lightningcss";

const require = createRequire(import.meta.url);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dryRun = process.argv.includes("--dry-run");
const iterations = parsePositiveInt(readArg("--iterations") ?? "3", "--iterations");
const samples = runBundlerSurfaceSnapshot();
const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-bundler-productization-"));

const MINIFY_PASS_IDS = [
  "comment-strip",
  "whitespace-strip",
  "number-compression",
  "color-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "calc-reduction",
  "print-css",
];

try {
  const preparedSamples = samples.samples.map((sample) => {
    const filePath = path.join(tempRoot, sample.path);
    writeFileSync(filePath, sample.source);
    return { ...sample, filePath };
  });

  const napiBinding = loadNapiBinding();
  const napiResults = measureLane({
    lane: "omena-napi-in-process",
    samples: preparedSamples,
    iterations: dryRun ? 1 : iterations,
    run: (sample) => runNapiBuild(napiBinding, sample),
  });

  const lightningResults = measureLane({
    lane: "lightningcss-node",
    samples: preparedSamples.filter((sample) => sample.dialect === "css"),
    iterations: dryRun ? 1 : iterations,
    run: runLightningCss,
  });

  const cliResults = dryRun
    ? plannedCliLane(preparedSamples)
    : measureLane({
        lane: "omena-cli-spawn",
        samples: preparedSamples,
        iterations,
        run: runCliBuild,
      });

  const report = {
    schemaVersion: "0",
    product: "omena-bundler-productization-benchmark",
    mode: dryRun ? "dry-run" : "measurement",
    speedClaimReady: false,
    timingPolicy: "raw-measurements-only-no-speed-claim",
    corpusSampleCount: preparedSamples.length,
    lanes: [napiResults, cliResults, lightningResults],
    measuredOperations: samples.measuredOperations,
    provenanceModes: ["source-map-on", "source-map-off"],
    memoryMetric: "rss-bytes",
  };

  assert.equal(samples.speedClaimReady, false);
  assert.ok(
    report.lanes.some((lane) => lane.lane === "omena-napi-in-process" && lane.sampleCount === 3),
    "benchmark must exercise the in-process NAPI lane over the full corpus",
  );
  assert.ok(
    report.lanes.some((lane) => lane.lane === "omena-cli-spawn"),
    "benchmark must include the CLI spawn lane or dry-run command plan",
  );
  assert.ok(
    report.lanes.some((lane) => lane.lane === "lightningcss-node" && lane.sampleCount >= 2),
    "benchmark must exercise the lightningcss CSS comparator lane",
  );

  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
}

function runBundlerSurfaceSnapshot() {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "bundler_productization_surface_snapshot",
      "--quiet",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.equal(result.status, 0, result.stderr);
  const snapshot = JSON.parse(result.stdout);
  assert.equal(snapshot.product, "omena-benchmarks.bundler-productization-surface");
  assert.equal(snapshot.speedClaimReady, false);
  assert.equal(snapshot.corpusSampleCount, 3);
  return snapshot;
}

function loadNapiBinding() {
  const candidates = ["@omena/napi", path.join(repoRoot, "rust/crates/omena-napi/pkg/index.js")];
  const failures = [];
  for (const candidate of candidates) {
    try {
      const binding = require(candidate);
      assert.equal(
        typeof binding.buildStyleSourcesWithContextJson,
        "function",
        `${candidate} must expose buildStyleSourcesWithContextJson`,
      );
      return binding;
    } catch (error) {
      failures.push(`${candidate}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
  throw new Error(`Unable to load @omena/napi for benchmark dry-run:\n${failures.join("\n")}`);
}

function measureLane({ lane, samples: laneSamples, iterations: laneIterations, run }) {
  const resultSamples = [];
  const rssBefore = process.memoryUsage().rss;
  for (const sample of laneSamples) {
    const measurements = [];
    let outputBytes = 0;
    for (let index = 0; index < laneIterations; index += 1) {
      const start = performance.now();
      const output = run(sample);
      const elapsedMs = performance.now() - start;
      outputBytes = output.byteLength;
      assert.ok(output.byteLength > 0, `${lane} should produce output for ${sample.name}`);
      measurements.push(Number(elapsedMs.toFixed(3)));
    }
    resultSamples.push({
      name: sample.name,
      path: sample.path,
      dialect: sample.dialect,
      outputBytes,
      elapsedMs: measurements,
    });
  }

  return {
    lane,
    sampleCount: resultSamples.length,
    iterations: laneIterations,
    rssDeltaBytes: process.memoryUsage().rss - rssBefore,
    samples: resultSamples,
  };
}

function runNapiBuild(binding, sample) {
  const summaryJson = binding.buildStyleSourcesWithContextJson(
    sample.filePath,
    JSON.stringify([{ stylePath: sample.filePath, styleSource: sample.source }]),
    MINIFY_PASS_IDS,
    "",
    "",
  );
  const summary = JSON.parse(summaryJson);
  const css = summary.execution?.outputCss;
  assert.equal(typeof css, "string", "NAPI summary must include execution.outputCss");
  return Buffer.from(css);
}

function runLightningCss(sample) {
  const result = lightningTransform({
    filename: sample.filePath,
    code: Buffer.from(sample.source),
    minify: true,
    sourceMap: true,
  });
  return result.code;
}

function runCliBuild(sample) {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--quiet",
      "--",
      "build",
      sample.filePath,
      "--minify",
      "--source-map",
      "--json",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.equal(result.status, 0, result.stderr);
  const summary = JSON.parse(result.stdout);
  const css = summary.execution?.outputCss;
  assert.equal(typeof css, "string", "CLI summary must include execution.outputCss");
  return Buffer.from(css);
}

function plannedCliLane(cliSamples) {
  return {
    lane: "omena-cli-spawn",
    sampleCount: cliSamples.length,
    iterations: 0,
    rssDeltaBytes: 0,
    dryRunOnly: true,
    commandShape: [
      "cargo",
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--quiet",
      "--",
      "build",
      "<sample>",
      "--minify",
      "--source-map",
      "--json",
    ],
    samples: cliSamples.map((sample) => ({
      name: sample.name,
      path: sample.path,
      dialect: sample.dialect,
    })),
  };
}

function readArg(name) {
  const prefix = `${name}=`;
  const inline = process.argv.find((arg) => arg.startsWith(prefix));
  if (inline) return inline.slice(prefix.length);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function parsePositiveInt(value, name) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}
