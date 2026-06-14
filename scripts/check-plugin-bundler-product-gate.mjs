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
const PRODUCT_GITHUB_REPO = "omenien/omena-css";
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
const REQUIRED_PRE_FREEZE_RUST_SYMBOLS = [
  {
    file: "rust/crates/omena-transform-bundle/src/lib.rs",
    symbols: [
      "TransformBundleEdgeV0",
      "TransformBundleAssetUrlV0",
      "TransformBundleAssetUrlRewriteSummaryV0",
      "TransformBundleChunkV0",
      "TransformBundleSourceSummaryV0",
    ],
  },
  {
    file: "rust/crates/omena-query-transform-runner/src/lib.rs",
    symbols: [
      "OmenaQueryTransformRunnerBoundaryV0",
      "OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0",
      "summarize_omena_query_transform_runner_boundary_v0",
    ],
  },
];

const gateEvidence = readGateEvidence(evidencePath);
const gateStatus = summarizeGateStatus(gateEvidence);

const prohibited = [
  ...findStandaloneBundlerSurfaces(repoRoot),
  ...findPrematureStableRustAliases(repoRoot),
  ...findPrematureRustBundlerV0Removal(repoRoot),
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
  validateReleaseCadenceConflictScope(
    parsed.releaseCadenceConflicts,
    collectReleaseCadenceConflictRepos(parsed),
    "releaseCadenceConflicts",
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
      symbols: [
        "TransformBundleEdge",
        "TransformBundleAssetUrl",
        "TransformBundleAssetUrlRewriteSummary",
        "TransformBundleChunk",
        "TransformBundleSourceSummary",
      ],
    },
    {
      file: path.join(rootDir, "rust", "crates", "omena-query-transform-runner", "src", "lib.rs"),
      symbols: ["OmenaQueryTransformRunnerBoundary"],
    },
  ];

  const hits = [];
  for (const check of checks) {
    if (!existsSync(check.file)) continue;
    const source = readFileSync(check.file, "utf8");
    for (const symbol of check.symbols) {
      if (sourceExposesPublicRustSymbol(source, symbol)) {
        hits.push(`${path.relative(rootDir, check.file)} exposes stable ${symbol}`);
      }
    }
  }
  return hits;
}

function findPrematureRustBundlerV0Removal(rootDir) {
  const hits = [];
  for (const requirement of REQUIRED_PRE_FREEZE_RUST_SYMBOLS) {
    const filePath = path.join(rootDir, requirement.file);
    if (!existsSync(filePath)) {
      hits.push(`${requirement.file} is missing before the bundler product gate fired`);
      continue;
    }
    const source = readFileSync(filePath, "utf8");
    for (const symbol of requirement.symbols) {
      if (!sourceContainsRustSymbol(source, symbol)) {
        hits.push(`${requirement.file} is missing pre-freeze V0 symbol ${symbol}`);
      }
    }
  }
  return hits;
}

function findBundlerFreezeClaims(rootDir) {
  const claims = [];
  for (const textPath of findRepoFilesByExtensions(rootDir, [".md", ".d.ts"])) {
    if (!isPublicFreezeClaimSurface(rootDir, textPath)) continue;
    const relativePath = path.relative(rootDir, textPath);
    const lines = readFileSync(textPath, "utf8").split(/\r?\n/);
    for (const [index, line] of lines.entries()) {
      if (isBundlerFreezeClaim(line)) {
        claims.push(`${relativePath}:${index + 1} claims bundler API freeze`);
      }
    }
  }
  for (const manifestPath of findRepoFiles(rootDir, "package.json")) {
    if (!isPublicPackageManifestSurface(rootDir, manifestPath)) continue;
    const relativePath = path.relative(rootDir, manifestPath);
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    for (const field of packageManifestFreezeClaimFields(manifest)) {
      claims.push(`${relativePath}:${field} claims bundler API freeze`);
    }
  }
  return claims;
}

function isPublicFreezeClaimSurface(rootDir, filePath) {
  const relativePath = path.relative(rootDir, filePath);
  const isMarkdown = relativePath.endsWith(".md");
  const isDeclaration = relativePath.endsWith(".d.ts");
  return (
    (isMarkdown &&
      (relativePath === "CHANGELOG.md" ||
        relativePath === "README.md" ||
        relativePath.startsWith(`docs${path.sep}`) ||
        relativePath.startsWith(`packages${path.sep}`))) ||
    (isDeclaration && relativePath.startsWith(`packages${path.sep}`))
  );
}

function isPublicPackageManifestSurface(rootDir, filePath) {
  const relativePath = path.relative(rootDir, filePath);
  return (
    relativePath.startsWith(`packages${path.sep}`) && path.basename(filePath) === "package.json"
  );
}

function packageManifestFreezeClaimFields(manifest) {
  const fields = [];
  if (typeof manifest.description === "string" && isBundlerFreezeClaim(manifest.description)) {
    fields.push("description");
  }
  if (
    Array.isArray(manifest.keywords) &&
    manifest.keywords.every((keyword) => typeof keyword === "string") &&
    isBundlerFreezeClaim(manifest.keywords.join(" "))
  ) {
    fields.push("keywords");
  }
  return fields;
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
    requireGitHubRepoBlobUrl(entry.evidenceUrl, `${label}.evidenceUrl`, repo);
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
    requireGitHubRepoBlobUrl(entry.evidenceUrl, `${label}.evidenceUrl`, repo);
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

function collectReleaseCadenceConflictRepos(evidence) {
  const repos = new Set([PRODUCT_GITHUB_REPO]);
  for (const entries of [evidence.externalNonMaintainerAdopters, evidence.moatDetachedAdopters]) {
    for (const entry of entries) {
      repos.add(requireRepo(entry.repo, "releaseCadenceConflictScope.repo"));
    }
  }
  return repos;
}

function validateReleaseCadenceConflictScope(entries, allowedRepos, fieldName) {
  for (const [index, entry] of entries.entries()) {
    const label = `${fieldName}[${index}].issueUrl`;
    const repo = releaseCadenceIssueRepo(entry.issueUrl, label);
    assert.equal(
      allowedRepos.has(repo),
      true,
      `${label} must belong to ${PRODUCT_GITHUB_REPO} or a validated adopter repository`,
    );
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

function requireGitHubRepoBlobUrl(value, label, repo) {
  const url = requireHttpUrl(value, label);
  const parsed = parseUrl(url, label);
  assert.equal(parsed.hostname, "github.com", `${label} must be a github.com URL`);
  assert.match(
    parsed.pathname.toLowerCase(),
    new RegExp(`^/${escapeRegExp(repo)}/blob/[^/]+/.+`, "u"),
    `${label} must point at a versioned file under ${repo}/blob/...`,
  );
  return url;
}

function requireBuildUrl(value, label, repo) {
  const url = requireHttpUrl(value, label);
  const parsed = parseUrl(url, label);
  if (parsed.hostname === "github.com") {
    assert.match(
      parsed.pathname.toLowerCase(),
      new RegExp(`^/${escapeRegExp(repo)}/actions/runs/[0-9]+/?$`, "u"),
      `${label} github.com build URL must point at ${repo}/actions/runs/{id}`,
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

function releaseCadenceIssueRepo(value, label) {
  const url = requireReleaseCadenceIssueUrl(value, label);
  const parsed = parseUrl(url, label);
  const match = /^\/([a-z0-9_.-]+)\/([a-z0-9_.-]+)\/issues\/[0-9]+\/?$/u.exec(
    parsed.pathname.toLowerCase(),
  );
  assert.ok(match, `${label} must point at a GitHub issue`);
  return `${match[1]}/${match[2]}`;
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
      /\bstable\b.*\bapi\b/.test(normalized) ||
      /\bapi\b.*\bstable\b/.test(normalized) ||
      /\bstable\b.*\bcontract\b/.test(normalized) ||
      /\bcontract\b.*\bstable\b/.test(normalized) ||
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
    isPublicFreezeClaimSurface("/repo", "/repo/packages/postcss-plugin/README.md"),
    true,
    "self-test: package READMEs are public freeze-claim surfaces",
  );
  assert.equal(
    isPublicFreezeClaimSurface("/repo", "/repo/packages/postcss-plugin/index.d.ts"),
    true,
    "self-test: package declarations are public freeze-claim surfaces",
  );
  assert.equal(
    isPublicFreezeClaimSurface("/repo", "/repo/.personal_docs/local.md"),
    false,
    "self-test: local planning docs are not public freeze-claim surfaces",
  );
  assert.deepEqual(
    packageManifestFreezeClaimFields({
      description: "Stable bundler API for production builds.",
      keywords: ["omena", "css"],
    }),
    ["description"],
    "self-test: package descriptions are guarded against premature freeze claims",
  );
  assert.deepEqual(
    packageManifestFreezeClaimFields({
      description: "Omena CSS integration.",
      keywords: ["bundler", "stable-api"],
    }),
    ["keywords"],
    "self-test: package keywords are guarded against premature freeze claims",
  );
  assert.equal(
    sourceContainsRustSymbol("pub struct TransformBundleEdgeV0 {}", "TransformBundleEdgeV0"),
    true,
    "self-test: Rust symbol scan finds a full identifier",
  );
  assert.equal(
    sourceContainsRustSymbol("pub struct TransformBundleEdgeV01 {}", "TransformBundleEdgeV0"),
    false,
    "self-test: Rust symbol scan does not match identifier prefixes",
  );
  assert.equal(
    sourceExposesPublicRustSymbol(
      "pub use crate::internal::TransformBundleEdgeV0 as TransformBundleEdge;",
      "TransformBundleEdge",
    ),
    true,
    "self-test: stable Rust re-export aliases are guarded",
  );
  assert.equal(
    sourceExposesPublicRustSymbol(
      "pub use crate::internal::TransformBundleEdge;",
      "TransformBundleEdge",
    ),
    true,
    "self-test: stable Rust direct re-exports are guarded",
  );
  assert.equal(
    sourceExposesPublicRustSymbol(
      "pub use crate::internal::TransformBundleEdgeV0;",
      "TransformBundleEdge",
    ),
    false,
    "self-test: V0 re-exports are not mistaken for stable Rust symbols",
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
    /versioned file/,
    "self-test: evidence URL must point at a versioned file in the declared adopter repo",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            evidenceUrl: "https://github.com/one/project/issues/1",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /versioned file/,
    "self-test: issue URLs cannot stand in for adopter code evidence",
  );
  assert.throws(
    () =>
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            evidenceUrl: "https://github.com/one/project/tree/main",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /versioned file/,
    "self-test: tree URLs cannot stand in for adopter code evidence",
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
      validateExternalNonMaintainerAdopters(
        [
          {
            ...externalAdopterFixture("one/project"),
            buildUrl: "https://github.com/one/project/actions/workflows/ci.yml",
          },
        ],
        "externalNonMaintainerAdopters",
      ),
    /actions\/runs/,
    "self-test: GitHub build evidence must point at a concrete run",
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
  validateReleaseCadenceConflictScope(
    [releaseConflictFixture("https://github.com/omenien/omena-css/issues/1")],
    new Set([PRODUCT_GITHUB_REPO]),
    "releaseCadenceConflicts",
  );
  validateReleaseCadenceConflictScope(
    [releaseConflictFixture("https://github.com/one/project/issues/1")],
    new Set([PRODUCT_GITHUB_REPO, "one/project"]),
    "releaseCadenceConflicts",
  );
  assert.throws(
    () =>
      validateReleaseCadenceConflictScope(
        [releaseConflictFixture("https://github.com/random/project/issues/1")],
        new Set([PRODUCT_GITHUB_REPO, "one/project"]),
        "releaseCadenceConflicts",
      ),
    /validated adopter repository/,
    "self-test: unrelated release-cadence issues cannot satisfy the cadence gate",
  );
}

function findRepoFiles(rootDir, fileName) {
  if (!existsSync(rootDir)) return [];
  const files = [];
  walkRepoFiles(rootDir, fileName, files);
  return files.toSorted();
}

function findRepoFilesByExtension(rootDir, extension) {
  if (!existsSync(rootDir)) return [];
  const files = [];
  walkRepoFilesByExtension(rootDir, extension, files);
  return files.toSorted();
}

function findRepoFilesByExtensions(rootDir, extensions) {
  return [
    ...new Set(extensions.flatMap((extension) => findRepoFilesByExtension(rootDir, extension))),
  ].toSorted();
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

function walkRepoFilesByExtension(dir, extension, files) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (shouldSkipScanDir(entry.name)) continue;
      walkRepoFilesByExtension(path.join(dir, entry.name), extension, files);
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(extension)) {
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

function sourceContainsRustSymbol(source, symbol) {
  return new RegExp(`\\b${escapeRegExp(symbol)}\\b`, "u").test(source);
}

function sourceExposesPublicRustSymbol(source, symbol) {
  const escapedSymbol = escapeRegExp(symbol);
  return [
    new RegExp(`\\bpub\\s+(?:struct|type|enum|trait)\\s+${escapedSymbol}\\b`, "u"),
    new RegExp(`\\bpub\\s+use\\b[^;\\n]*\\bas\\s+${escapedSymbol}\\b`, "u"),
    new RegExp(`\\bpub\\s+use\\b[^;\\n]*\\b${escapedSymbol}\\b\\s*;`, "u"),
  ].some((pattern) => pattern.test(source));
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
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
