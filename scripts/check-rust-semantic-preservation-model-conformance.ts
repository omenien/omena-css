import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

interface SemanticPreservationModelConformanceV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly cascadeSeedProduct: string;
  readonly cascadeSeedCaseCount: number;
  readonly cascadeSeedFailedCount: number;
  readonly cascadeSeedDigest: string;
  readonly wptSeedProduct: string;
  readonly wptSeedCaseCount: number;
  readonly wptSeedFailedCount: number;
  readonly wptSeedDigest: string;
  readonly semanticObservationCaseCount: number;
  readonly semanticObservationFailedCount: number;
  readonly modelConformancePassed: boolean;
}

const repoRoot = process.cwd();
const artifactPath = path.join(
  repoRoot,
  "rust/crates/omena-transform-passes/fixtures/semantic-preservation/model-conformance.json",
);
const artifactSource = readFileSync(artifactPath, "utf8");
const artifact = JSON.parse(artifactSource) as SemanticPreservationModelConformanceV0;

assert.equal(artifact.schemaVersion, "0");
assert.equal(artifact.product, "omena-transform-passes.semantic-preservation-model-conformance");
assert.equal(artifact.cascadeSeedProduct, "omena-cascade.conformance-seed-corpus");
assert.ok(artifact.cascadeSeedCaseCount > 0, "cascade seed corpus must not be empty");
assert.equal(artifact.cascadeSeedFailedCount, 0);
assert.match(artifact.cascadeSeedDigest, /^fnv1a64:[0-9a-f]{16}$/u);
assert.equal(artifact.wptSeedProduct, "omena-cascade.wpt-cascade-seed-corpus");
assert.ok(artifact.wptSeedCaseCount >= 200, "WPT seed corpus coverage regressed");
assert.equal(artifact.wptSeedFailedCount, 0);
assert.match(artifact.wptSeedDigest, /^fnv1a64:[0-9a-f]{16}$/u);
assert.ok(
  artifact.semanticObservationCaseCount > 0,
  "semantic observation conformance cases must not be empty",
);
assert.equal(artifact.semanticObservationFailedCount, 0);
assert.equal(artifact.modelConformancePassed, true);

const rustGate = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-passes",
    "semantic_preservation_model_conformance_report_matches_committed_artifact",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
assert.equal(
  rustGate.status,
  0,
  `semantic preservation model conformance gate failed\nstdout=${rustGate.stdout}\nstderr=${rustGate.stderr}`,
);
assertCargoTestExecuted(rustGate, "semantic preservation model conformance gate");

process.stdout.write(
  JSON.stringify(
    {
      product: "omena-transform-passes.semantic-preservation-model-conformance.check",
      artifactPath: path.relative(repoRoot, artifactPath),
      artifactSha256: createHash("sha256").update(artifactSource).digest("hex"),
      cascadeSeedCaseCount: artifact.cascadeSeedCaseCount,
      wptSeedCaseCount: artifact.wptSeedCaseCount,
      semanticObservationCaseCount: artifact.semanticObservationCaseCount,
      rustGatePassed: true,
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function assertCargoTestExecuted(result: ReturnType<typeof spawnSync>, label: string): void {
  const output = `${result.stdout}\n${result.stderr}`;
  const passedCounts = [...output.matchAll(/test result: ok\. (\d+) passed;/gu)].map((match) =>
    Number(match[1]),
  );
  assert.ok(
    passedCounts.some((count) => count > 0),
    `${label} matched zero Rust tests\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
}
