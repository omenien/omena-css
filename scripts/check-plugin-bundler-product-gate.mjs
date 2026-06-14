#!/usr/bin/env node
import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const evidencePath = path.join(repoRoot, "docs", "bundler-product-gate.json");
const VALID_ADOPTER_SURFACES = new Set([
  "css-build-adapter",
  "postcss-plugin",
  "vite-plugin",
  "omena-cli-build",
]);
const MAINTAINER_GITHUB_OWNERS = new Set(["omenien", "yongsk0066"]);
const REPO_SCAN_IGNORED_DIRS = new Set([
  ".cache",
  ".git",
  ".next",
  ".personal_docs",
  ".turbo",
  "coverage",
  "dist",
  "node_modules",
  "out",
  "target",
]);

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
  validateExternalNonMaintainerAdopters(
    parsed.externalNonMaintainerAdopters,
    "externalNonMaintainerAdopters",
  );
  validateMoatDetachedAdopters(parsed.moatDetachedAdopters, "moatDetachedAdopters");
  validateReleaseCadenceConflicts(parsed.releaseCadenceConflicts, "releaseCadenceConflicts");
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

  for (const manifestPath of findRepoFiles(rootDir, "package.json")) {
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    if (manifest.name === "@omena/bundler") {
      hits.push(`${path.relative(rootDir, manifestPath)} declares @omena/bundler`);
    }
  }

  const rustRoot = path.join(rootDir, "rust");
  for (const manifestPath of findRepoFiles(rustRoot, "Cargo.toml")) {
    const source = readFileSync(manifestPath, "utf8");
    if (readCargoPackageName(source) === "omena-bundler") {
      hits.push(`${path.relative(rootDir, manifestPath)} declares package name omena-bundler`);
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

function validateExternalNonMaintainerAdopters(entries, fieldName) {
  const repos = new Set();
  for (const [index, entry] of entries.entries()) {
    const label = `${fieldName}[${index}]`;
    assertRecord(entry, label);
    const repo = requireRepo(entry.repo, `${label}.repo`);
    rejectMaintainerOwnedRepo(repo, `${label}.repo`);
    assert.equal(
      repos.has(repo),
      false,
      `${label}.repo duplicates ${repo}; external adopter evidence must be counted by unique repository`,
    );
    repos.add(repo);
    assert.equal(
      entry.maintainerRelation,
      "non-maintainer",
      `${label}.maintainerRelation must be "non-maintainer"`,
    );
    requireGitHubRepoUrl(entry.evidenceUrl, `${label}.evidenceUrl`, repo);
    requireBuildUrl(entry.buildUrl, `${label}.buildUrl`, repo);
    requireAdopterSurface(entry.surface, `${label}.surface`);
  }
}

function validateMoatDetachedAdopters(entries, fieldName) {
  const repos = new Set();
  for (const [index, entry] of entries.entries()) {
    const label = `${fieldName}[${index}]`;
    assertRecord(entry, label);
    const repo = requireRepo(entry.repo, `${label}.repo`);
    rejectMaintainerOwnedRepo(repo, `${label}.repo`);
    assert.equal(
      repos.has(repo),
      false,
      `${label}.repo duplicates ${repo}; moat-detached evidence must be counted by unique repository`,
    );
    repos.add(repo);
    assert.equal(
      entry.usesEditorCheckerMoat,
      false,
      `${label}.usesEditorCheckerMoat must be false`,
    );
    requireGitHubRepoUrl(entry.evidenceUrl, `${label}.evidenceUrl`, repo);
    requireBuildUrl(entry.buildUrl, `${label}.buildUrl`, repo);
    requireAdopterSurface(entry.surface, `${label}.surface`);
  }
}

function validateReleaseCadenceConflicts(entries, fieldName) {
  const issueUrls = new Set();
  for (const [index, entry] of entries.entries()) {
    const label = `${fieldName}[${index}]`;
    assertRecord(entry, label);
    const issueUrl = requireReleaseCadenceIssueUrl(entry.issueUrl, `${label}.issueUrl`);
    assert.equal(
      issueUrls.has(issueUrl),
      false,
      `${label}.issueUrl duplicates ${issueUrl}; release-cadence conflicts must be unique`,
    );
    issueUrls.add(issueUrl);
    assert.equal(
      entry.conflictKind,
      "release-cadence",
      `${label}.conflictKind must be "release-cadence"`,
    );
    requireNonEmptyString(entry.summary, `${label}.summary`);
  }
}

function assertRecord(value, label) {
  assert.equal(typeof value, "object", `${label} must be an object`);
  assert.notEqual(value, null, `${label} must not be null`);
  assert.equal(Array.isArray(value), false, `${label} must not be an array`);
}

function requireRepo(value, label) {
  const repo = requireNonEmptyString(value, label).toLowerCase();
  assert.match(
    repo,
    /^[a-z0-9_.-]+\/[a-z0-9_.-]+$/,
    `${label} must be a GitHub repository in owner/name form`,
  );
  return repo;
}

function rejectMaintainerOwnedRepo(repo, label) {
  const owner = repo.split("/")[0];
  assert.equal(
    MAINTAINER_GITHUB_OWNERS.has(owner),
    false,
    `${label} must not be under a known maintainer-owned GitHub owner: ${[
      ...MAINTAINER_GITHUB_OWNERS,
    ].join(", ")}`,
  );
}

function requireAdopterSurface(value, label) {
  const surface = requireNonEmptyString(value, label);
  assert.equal(
    VALID_ADOPTER_SURFACES.has(surface),
    true,
    `${label} must be one of: ${[...VALID_ADOPTER_SURFACES].join(", ")}`,
  );
  return surface;
}

function requireHttpUrl(value, label) {
  const url = requireNonEmptyString(value, label);
  const parsed = parseUrl(url, label);
  assert.equal(parsed.protocol, "https:", `${label} must be an https URL`);
  return url;
}

function requireGitHubRepoUrl(value, label, repo) {
  const url = requireHttpUrl(value, label);
  const parsed = parseUrl(url, label);
  assert.equal(parsed.hostname, "github.com", `${label} must be a github.com URL`);
  assert.equal(
    parsed.pathname.toLowerCase().startsWith(`/${repo}/`),
    true,
    `${label} must point inside the declared repository ${repo}`,
  );
  return url;
}

function requireBuildUrl(value, label, repo) {
  const url = requireHttpUrl(value, label);
  const parsed = parseUrl(url, label);
  if (parsed.hostname === "github.com") {
    assert.equal(
      parsed.pathname.toLowerCase().startsWith(`/${repo}/actions/`),
      true,
      `${label} github.com build URL must point at ${repo}/actions`,
    );
  }
  return url;
}

function requireReleaseCadenceIssueUrl(value, label) {
  const url = requireHttpUrl(value, label);
  const parsed = parseUrl(url, label);
  assert.equal(parsed.hostname, "github.com", `${label} must be a github.com issue URL`);
  assert.match(
    parsed.pathname.toLowerCase(),
    /^\/[a-z0-9_.-]+\/[a-z0-9_.-]+\/issues\/[0-9]+\/?$/u,
    `${label} must point at a GitHub issue`,
  );
  return url;
}

function parseUrl(value, label) {
  try {
    return new URL(value);
  } catch (error) {
    assert.fail(`${label} must be a valid URL: ${error.message}`);
  }
}

function requireNonEmptyString(value, label) {
  assert.equal(typeof value, "string", `${label} must be a string`);
  assert.notEqual(value.trim(), "", `${label} must not be empty`);
  return value.trim();
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
  const evidence = firedEvidence();
  const fired = summarizeGateStatus(evidence);
  assert.equal(fired.allFired, true, "self-test: all three product gates fire together");
  validateExternalNonMaintainerAdopters(
    evidence.externalNonMaintainerAdopters,
    "externalNonMaintainerAdopters",
  );
  validateMoatDetachedAdopters(evidence.moatDetachedAdopters, "moatDetachedAdopters");
  validateReleaseCadenceConflicts(evidence.releaseCadenceConflicts, "releaseCadenceConflicts");

  const notFired = summarizeGateStatus({
    externalNonMaintainerAdopters: [
      externalAdopterFixture("one/project"),
      externalAdopterFixture("two/project"),
    ],
    moatDetachedAdopters: [moatDetachedFixture("one/project")],
    releaseCadenceConflicts: [
      releaseConflictFixture("https://github.com/omenien/omena-css/issues/1"),
    ],
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
  assert.equal(
    readCargoPackageName("[workspace]\nmembers = []\n"),
    null,
    "self-test: workspace-only Cargo manifest is not a package",
  );
  assert.equal(
    readCargoPackageName('[package]\nname = "omena-bundler"\nversion = "0.0.0"\n'),
    "omena-bundler",
    "self-test: Cargo package name is read from the package section",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            maintainerRelation: "same-maintainer",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /maintainerRelation/,
    "self-test: same-maintainer evidence cannot satisfy the external adopter gate",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [externalAdopterFixture("omenien/adopter-smoke")],
        "externalNonMaintainerAdopters",
      ),
    /known maintainer-owned/,
    "self-test: maintainer-owned repositories cannot satisfy the external adopter gate",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            evidenceUrl: "https://github.com/two/project/blob/main/package.json",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /declared repository/,
    "self-test: evidence URL must point inside the declared adopter repo",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            buildUrl: "https://github.com/two/project/actions/runs/1",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /github.com build URL/,
    "self-test: GitHub build URL must point at the declared adopter repo",
  );
  assert.throws(
    () =>
      validateMoatDetachedAdopters(
        [{ ...moatDetachedFixture("one/project"), usesEditorCheckerMoat: true }],
        "moatDetachedAdopters",
      ),
    /usesEditorCheckerMoat/,
    "self-test: moat-attached evidence cannot satisfy the moat-detached gate",
  );
  assert.throws(
    () =>
      validateMoatDetachedAdopters(
        [moatDetachedFixture("yongsk0066/adopter-smoke")],
        "moatDetachedAdopters",
      ),
    /known maintainer-owned/,
    "self-test: maintainer-owned repositories cannot satisfy the moat-detached gate",
  );
  assert.throws(
    () =>
      validateReleaseCadenceConflicts(
        [
          {
            ...releaseConflictFixture("https://example.com/issues/1"),
          },
        ],
        "releaseCadenceConflicts",
      ),
    /github.com issue URL/,
    "self-test: release-cadence conflict evidence must point at a GitHub issue",
  );
  assert.throws(
    () =>
      validateReleaseCadenceConflicts(
        [
          {
            ...releaseConflictFixture("https://github.com/omenien/omena-css/issues/1"),
            conflictKind: "feature-request",
          },
        ],
        "releaseCadenceConflicts",
      ),
    /conflictKind/,
    "self-test: non-release-cadence issues cannot satisfy the cadence gate",
  );
}

function findRepoFiles(rootDir, fileName) {
  if (!existsSync(rootDir)) return [];
  const files = [];
  walkRepoFiles(rootDir, fileName, files);
  return files.toSorted();
}

function walkRepoFiles(dir, fileName, files) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (shouldSkipScanDir(entry.name)) continue;
      walkRepoFiles(path.join(dir, entry.name), fileName, files);
      continue;
    }
    if (entry.isFile() && entry.name === fileName) {
      files.push(path.join(dir, entry.name));
    }
  }
}

function shouldSkipScanDir(name) {
  return REPO_SCAN_IGNORED_DIRS.has(name);
}

function readCargoPackageName(source) {
  let inPackage = false;
  for (const line of source.split(/\r?\n/u)) {
    const trimmed = line.trim();
    if (trimmed === "[package]") {
      inPackage = true;
      continue;
    }
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      inPackage = false;
      continue;
    }
    if (!inPackage) continue;
    const match = /^name\s*=\s*"([^"]+)"$/u.exec(trimmed);
    if (match) return match[1];
  }
  return null;
}

function firedEvidence() {
  return {
    externalNonMaintainerAdopters: [
      externalAdopterFixture("one/project"),
      externalAdopterFixture("two/project"),
      externalAdopterFixture("three/project"),
    ],
    moatDetachedAdopters: [moatDetachedFixture("one/project")],
    releaseCadenceConflicts: [
      releaseConflictFixture("https://github.com/omenien/omena-css/issues/1"),
    ],
  };
}

function externalAdopterFixture(repo) {
  return {
    repo,
    maintainerRelation: "non-maintainer",
    surface: "postcss-plugin",
    evidenceUrl: `https://github.com/${repo}/blob/main/package.json`,
    buildUrl: `https://github.com/${repo}/actions/runs/1`,
  };
}

function moatDetachedFixture(repo) {
  return {
    repo,
    usesEditorCheckerMoat: false,
    surface: "postcss-plugin",
    evidenceUrl: `https://github.com/${repo}/blob/main/package.json`,
    buildUrl: `https://github.com/${repo}/actions/runs/1`,
  };
}

function releaseConflictFixture(issueUrl) {
  return {
    issueUrl,
    conflictKind: "release-cadence",
    summary: "Bundler surface needs to ship independently of the lockstep train.",
  };
}
