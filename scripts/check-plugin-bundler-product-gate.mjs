#!/usr/bin/env node
import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const evidencePath = path.join(repoRoot, "docs", "bundler-product-gate.json");

const gateEvidence = readGateEvidence(evidencePath);
const gateStatus = summarizeGateStatus(gateEvidence);

const prohibited = [
  ...findStandaloneBundlerSurfaces(repoRoot),
  ...findPrematureStableRustAliases(repoRoot),
  ...findBundlerFreezeClaims(repoRoot),
];

if (!gateStatus.allFired) {
  assert.equal(
    prohibited.length,
    0,
    [
      "bundler product gate is not fired, but product-only bundler surface changes were found:",
      ...prohibited.map((item) => `  - ${item}`),
      "",
      "Keep bundler behavior as an omena-css mode until the product gate has evidence for:",
      "  - at least 3 external non-maintainer adopters",
      "  - at least 1 moat-detached adopter",
      "  - at least 1 release-cadence conflict",
    ].join("\n"),
  );
}

selfTest();

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-css.bundler-product-gate",
      evidenceFilePresent: existsSync(evidencePath),
      externalNonMaintainerAdopters: gateStatus.externalNonMaintainerAdopters,
      moatDetachedAdopters: gateStatus.moatDetachedAdopters,
      releaseCadenceConflicts: gateStatus.releaseCadenceConflicts,
      allProductGatesFired: gateStatus.allFired,
      prohibitedProductSurfaces: prohibited.length,
    },
    null,
    2,
  )}\n`,
);

function readGateEvidence(filePath) {
  if (!existsSync(filePath)) {
    return {
      schemaVersion: "0",
      externalNonMaintainerAdopters: [],
      moatDetachedAdopters: [],
      releaseCadenceConflicts: [],
    };
  }

  const parsed = JSON.parse(readFileSync(filePath, "utf8"));
  assert.equal(parsed.schemaVersion, "0", `${filePath} must use schemaVersion "0"`);
  assert.ok(
    Array.isArray(parsed.externalNonMaintainerAdopters),
    `${filePath} must declare externalNonMaintainerAdopters`,
  );
  assert.ok(
    Array.isArray(parsed.moatDetachedAdopters),
    `${filePath} must declare moatDetachedAdopters`,
  );
  assert.ok(
    Array.isArray(parsed.releaseCadenceConflicts),
    `${filePath} must declare releaseCadenceConflicts`,
  );
  return parsed;
}

function summarizeGateStatus(evidence) {
  const externalNonMaintainerAdopters = evidence.externalNonMaintainerAdopters.length;
  const moatDetachedAdopters = evidence.moatDetachedAdopters.length;
  const releaseCadenceConflicts = evidence.releaseCadenceConflicts.length;
  return {
    externalNonMaintainerAdopters,
    moatDetachedAdopters,
    releaseCadenceConflicts,
    allFired:
      externalNonMaintainerAdopters >= 3 &&
      moatDetachedAdopters >= 1 &&
      releaseCadenceConflicts >= 1,
  };
}

function findStandaloneBundlerSurfaces(rootDir) {
  const hits = [];
  const rustBundlerDir = path.join(rootDir, "rust", "crates", "omena-bundler");
  if (existsSync(rustBundlerDir)) {
    hits.push("rust/crates/omena-bundler exists");
  }

  const packageRoot = path.join(rootDir, "packages");
  if (existsSync(packageRoot)) {
    for (const dirName of readdirSync(packageRoot).toSorted()) {
      const manifestPath = path.join(packageRoot, dirName, "package.json");
      if (!existsSync(manifestPath)) continue;
      const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
      if (manifest.name === "@omena/bundler") {
        hits.push(`packages/${dirName}/package.json declares @omena/bundler`);
      }
    }
  }

  return hits;
}

function findPrematureStableRustAliases(rootDir) {
  const checks = [
    {
      file: path.join(rootDir, "rust", "crates", "omena-transform-bundle", "src", "lib.rs"),
      pattern:
        /\bpub\s+(?:struct|type)\s+(TransformBundle(?:Edge|AssetUrl|AssetUrlRewriteSummary|Chunk|SourceSummary))\b/g,
    },
    {
      file: path.join(rootDir, "rust", "crates", "omena-query-transform-runner", "src", "lib.rs"),
      pattern: /\bpub\s+(?:struct|type)\s+(OmenaQueryTransformRunnerBoundary)\b/g,
    },
  ];

  const hits = [];
  for (const check of checks) {
    if (!existsSync(check.file)) continue;
    const source = readFileSync(check.file, "utf8");
    for (const match of source.matchAll(check.pattern)) {
      hits.push(`${path.relative(rootDir, check.file)} exposes stable ${match[1]}`);
    }
  }
  return hits;
}

function findBundlerFreezeClaims(rootDir) {
  const changelogPath = path.join(rootDir, "CHANGELOG.md");
  if (!existsSync(changelogPath)) return [];

  const claims = [];
  const lines = readFileSync(changelogPath, "utf8").split(/\r?\n/);
  for (const [index, line] of lines.entries()) {
    if (isBundlerFreezeClaim(line)) {
      claims.push(`CHANGELOG.md:${index + 1} claims bundler API freeze`);
    }
  }
  return claims;
}

function isBundlerFreezeClaim(line) {
  const normalized = line.toLowerCase();
  return (
    normalized.includes("bundler") &&
    (normalized.includes("api freeze") ||
      normalized.includes("frozen api") ||
      normalized.includes("stable api") ||
      normalized.includes("stable contract") ||
      /\bv0\b.*\bremoved\b/.test(normalized) ||
      /\bremoved\b.*\bv0\b/.test(normalized))
  );
}

function selfTest() {
  const fired = summarizeGateStatus({
    externalNonMaintainerAdopters: [{ repo: "a" }, { repo: "b" }, { repo: "c" }],
    moatDetachedAdopters: [{ repo: "a" }],
    releaseCadenceConflicts: [{ issue: "x" }],
  });
  assert.equal(fired.allFired, true, "self-test: all three product gates fire together");

  const notFired = summarizeGateStatus({
    externalNonMaintainerAdopters: [{ repo: "a" }, { repo: "b" }],
    moatDetachedAdopters: [{ repo: "a" }],
    releaseCadenceConflicts: [{ issue: "x" }],
  });
  assert.equal(
    notFired.allFired,
    false,
    "self-test: two external non-maintainer adopters are not enough",
  );

  assert.equal(
    isBundlerFreezeClaim("- Bundler API freeze removes V0 from the public surface."),
    true,
    "self-test: bundler API freeze wording is guarded",
  );
  assert.equal(
    isBundlerFreezeClaim("- No public Cargo API freeze claim."),
    false,
    "self-test: non-bundler freeze wording is not guarded",
  );
}
