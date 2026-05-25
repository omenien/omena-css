import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

type TheoryClaimGuardSummary = {
  readonly schemaVersion: "0";
  readonly product: "rust.m4-closure-audit";
  readonly m4Complete: boolean;
  readonly theoryClaimGuard: {
    readonly ladder: readonly string[];
    readonly stages: Record<string, readonly unknown[]>;
    readonly summary: Record<string, number>;
  };
};

type ReleaseDisposition = {
  readonly surface: string;
  readonly disposition:
    | "publicReleaseClaim"
    | "v0ApiFreezeCandidate"
    | "internalSubstrate"
    | "releaseBoundary"
    | "deferred";
  readonly evidence: readonly string[];
};

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  readonly version: string;
  readonly scripts: Record<string, string>;
};
const changelog = read("CHANGELOG.md");
const unreleased = extractSection(changelog, "## [Unreleased]");
const releasing = read("RELEASING.md");
const cargoToml = read("rust/Cargo.toml");
const publishScript = read("scripts/publish-extension.sh");

const M5_AUDIT_TARGET = "release/check/release-m5-api-freeze-audit";
const M5_AUDIT_SCRIPT = "check:release-m5-api-freeze-audit";

const dispositionTable: readonly ReleaseDisposition[] = [
  {
    surface: "ClassValueUniverseProviderV0",
    disposition: "publicReleaseClaim",
    evidence: [
      "server/engine-core-ts/src/core/binder/class-value-universe-provider.ts",
      "server/engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin.ts",
      "server/engine-core-ts/src/core/binder/cva-recipe-plugin.ts",
      "test/unit/binder/binder-plugin.test.ts",
      "test/unit/query/read-domain-class-references.test.ts",
      "test/unit/abstract-value/selector-projection.test.ts",
    ],
  },
  {
    surface: "LinearProvenanceV0<K>",
    disposition: "v0ApiFreezeCandidate",
    evidence: [
      "rust/crates/omena-abstract-value/src/types.rs",
      "rust/crates/omena-query/src/types.rs",
      "scripts/check-rust-m4-closure-audit.ts",
    ],
  },
  {
    surface: "DatalogRuleEvaluatorV0",
    disposition: "v0ApiFreezeCandidate",
    evidence: ["rust/crates/omena-incremental/src/lib.rs"],
  },
  {
    surface: "ModalCheckWitnessV0",
    disposition: "v0ApiFreezeCandidate",
    evidence: ["rust/crates/omena-cascade/src/model.rs", "rust/crates/omena-cascade/src/modal.rs"],
  },
  {
    surface: "BeliefPropagationIterationV0",
    disposition: "v0ApiFreezeCandidate",
    evidence: [
      "rust/crates/omena-abstract-value/src/types.rs",
      "rust/crates/omena-abstract-value/src/reduced_product.rs",
    ],
  },
  {
    surface: "CascadeMarginSchemaV0",
    disposition: "internalSubstrate",
    evidence: [
      "rust/crates/omena-cascade/src/model.rs",
      "rust/crates/omena-cascade/src/ranking.rs",
    ],
  },
  {
    surface: "FastFactsV0/AnalyzedGraphV0",
    disposition: "internalSubstrate",
    evidence: [
      "rust/crates/omena-query/src/types.rs",
      "rust/crates/omena-query/src/style/substrate.rs",
    ],
  },
  {
    surface: "docs19/20 automation and omena-testkit/cme-checker",
    disposition: "releaseBoundary",
    evidence: [
      "rust/crates/omena-testkit/src/boundary.rs",
      "rust/crates/omena-testkit/src/snapshot.rs",
      "scripts/check-rust-m4-axis-a-closure-audit.ts",
      "scripts/check-cme-checker-testkit-archetypes.ts",
      "packages/cme-checker/src/testkit.ts",
    ],
  },
  {
    surface: "Cargo workspace version",
    disposition: "releaseBoundary",
    evidence: ["rust/Cargo.toml", "RELEASING.md"],
  },
  {
    surface: "Marketplace/Open VSX publish",
    disposition: "deferred",
    evidence: ["scripts/publish-extension.sh", "RELEASING.md"],
  },
  {
    surface: "Issue #61 broader RFC",
    disposition: "deferred",
    evidence: ["RELEASING.md"],
  },
];

assertSemver(packageJson.version);
assert.equal(cargoToml.match(/^version = "([^"]+)"/m)?.[1], "0.2.0");
assert.match(cargoToml, /^publish = false$/m);

assertIncludes(packageJson.scripts[M5_AUDIT_SCRIPT], "check-m5-release-api-freeze-audit.ts");
assertIncludes(packageJson.scripts.package, M5_AUDIT_TARGET);
assertIncludes(packageJson.scripts.package, "package-extension-vsix.ts");
assertIncludes(packageJson.scripts["release:verify"], M5_AUDIT_TARGET);
assertIncludes(publishScript, `pnpm ${M5_AUDIT_SCRIPT}`);
assertIncludes(publishScript, "package-extension-vsix.ts");
assertIncludes(publishScript, "vsce publish --packagePath");
assertIncludes(
  packageJson.scripts.package,
  "release/check/packaged-omena-lsp-server-type-fact-protocol",
);
assertIncludes(publishScript, "pnpm check:packaged-omena-lsp-server-type-fact-protocol");

assertNoInternalMilestoneJargon(unreleased);
assertIncludes(unreleased, "Variant recipe class-value substrate");
assertIncludes(unreleased, "ClassValueUniverseProviderV0");
assertIncludes(unreleased, "vanilla-extract recipes");
assertIncludes(unreleased, "cva phase 1");
assertIncludes(unreleased, "without introducing a\n  public plugin ABI");
assertIncludes(unreleased, "V0 theory contract substrate");
assertIncludes(
  unreleased,
  "keeping Datalog host, modal theorem, belief-propagation paper, and safety-margin\n  claims out of public release wording",
);
assertIncludes(unreleased, "staged research contracts");
assertIncludes(unreleased, "final APIs or completed theory claims");

assertIncludes(releasing, "Release claim discipline");
assertIncludes(releasing, "pnpm check:release-m5-api-freeze-audit");
assertIncludes(releasing, "release/API-freeze wording\ngate");
assertIncludes(releasing, "Avoid internal milestone labels, planning shorthand, and P-numbering");
assertIncludes(
  releasing,
  "For issue #61, release text may mention only the Finding-D class-value-universe",
);
assertIncludes(releasing, "Do not close or describe the broader #61 resolver/Sass/");
assertIncludes(
  releasing,
  "Automation and testkit surfaces are release-framed only when their fixture",
);
assertIncludes(releasing, "Cargo crate versioning stays on the gradual `0.2.x` line");
assertIncludes(releasing, "Do not publish or describe a Cargo `1.0.0` API-freeze line");

assertEvidenceMarkers();
const theoryAudit = loadTheoryClaimAudit();
assertTheoryClaimAudit(theoryAudit);

process.stdout.write(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.m5-api-freeze-audit",
      packageVersion: packageJson.version,
      cargoWorkspaceVersion: "0.2.0",
      releaseDisposition: dispositionTable,
      theoryClaimGuardSummary: theoryAudit.theoryClaimGuard.summary,
      publishPath: {
        auditGate: M5_AUDIT_TARGET,
        packagedTypeFactProtocolGate: "release/check/packaged-omena-lsp-server-type-fact-protocol",
        marketplacePublishInvoked: false,
        openVsxPublishInvoked: false,
      },
      issue61: {
        findingDReleaseClaimAllowed: true,
        broaderRfcReleaseClaimAllowed: false,
      },
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function assertIncludes(source: string | undefined, marker: string): void {
  assert.equal(typeof source, "string", `expected source containing ${marker}`);
  assert.ok(source.includes(marker), `missing marker: ${marker}`);
}

function assertSemver(version: string): void {
  assert.match(version, /^\d+\.\d+\.\d+$/, "extension release version must be stable semver");
}

function extractSection(source: string, heading: string): string {
  const start = source.indexOf(heading);
  assert.notEqual(start, -1, `missing section ${heading}`);
  const next = source.indexOf("\n## ", start + heading.length);
  return source.slice(start, next === -1 ? source.length : next);
}

function assertNoInternalMilestoneJargon(source: string): void {
  const forbiddenPatterns = [
    /\bM[0-9]+(?:-[a-z]+)?\b/,
    /\bm4-(?:alpha|beta|gamma)\b/i,
    /\bZ[0-9]+\b/,
    /\bP[0-9]{1,3}\b/,
  ];
  for (const pattern of forbiddenPatterns) {
    assert.equal(source.match(pattern)?.[0], undefined, `release text contains ${pattern}`);
  }
}

function assertEvidenceMarkers(): void {
  const markers: Record<string, readonly string[]> = {
    "server/engine-core-ts/src/core/binder/class-value-universe-provider.ts": [
      "export interface ClassValueUniverseProviderV0",
    ],
    "server/engine-core-ts/src/core/abstract-value/class-value-universe.ts": [
      'kind: "reduced-product"',
      "reducedProductClassValueUniverseV0",
    ],
    "server/engine-core-ts/src/core/binder/variant-recipe-universe.ts": [
      "base",
      "compoundVariants",
      "defaultVariants",
      "ClassValueUniverseProviderV0",
    ],
    "server/engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin.ts": [
      "vanillaExtractRecipeClassValueUniverseProviderV0",
    ],
    "server/engine-core-ts/src/core/binder/cva-recipe-plugin.ts": [
      "cvaRecipeClassValueUniverseProviderV0",
    ],
    "rust/crates/omena-abstract-value/src/types.rs": [
      "pub struct LinearProvenanceV0",
      "pub struct BeliefPropagationIterationV0",
    ],
    "rust/crates/omena-incremental/src/lib.rs": ["pub struct DatalogRuleEvaluatorV0"],
    "rust/crates/omena-cascade/src/model.rs": [
      "pub struct ModalCheckWitnessV0",
      "pub struct CascadeMarginSchemaV0",
    ],
    "rust/crates/omena-query/src/types.rs": [
      "pub struct FastFactsV0",
      "pub struct AnalyzedGraphV0",
    ],
    "rust/crates/omena-testkit/src/boundary.rs": [
      "schema_version",
      "shared-fixture-parser",
      "snapshotGovernanceKnownFailurePolicy",
    ],
    "rust/crates/omena-testkit/src/snapshot.rs": [
      "snapshot_manifest_schema",
      "known_failure_schema",
    ],
    "scripts/check-rust-m4-axis-a-closure-audit.ts": [
      "known-failure",
      "omena-testkit",
      "cme-checker",
    ],
    "scripts/check-cme-checker-testkit-archetypes.ts": ['schemaVersion: "0"', "omena-testkit"],
    "packages/cme-checker/src/testkit.ts": [
      "CmeCheckerTestkitArchetypeV0",
      "source-missing",
      "style-unused",
      "style-recovery",
    ],
  };

  for (const [relativePath, requiredMarkers] of Object.entries(markers)) {
    const source = read(relativePath);
    for (const marker of requiredMarkers) {
      assertIncludes(source, marker);
    }
  }

  for (const row of dispositionTable) {
    for (const evidencePath of row.evidence) {
      read(evidencePath);
    }
  }
}

function loadTheoryClaimAudit(): TheoryClaimGuardSummary {
  const output = execFileSync(
    process.execPath,
    ["--import", "tsx", "./scripts/check-rust-m4-closure-audit.ts"],
    {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  return JSON.parse(output) as TheoryClaimGuardSummary;
}

function assertTheoryClaimAudit(audit: TheoryClaimGuardSummary): void {
  assert.equal(audit.schemaVersion, "0");
  assert.equal(audit.product, "rust.m4-closure-audit");
  assert.equal(audit.m4Complete, true);
  assert.deepEqual(audit.theoryClaimGuard.ladder, [
    "descriptorOnly",
    "fixtureRecordOnly",
    "partialPropertyTest",
    "propertyTestEnforced",
  ]);
  for (const stageName of ["m4-alpha", "m4-beta", "m4-gamma"]) {
    assert.ok(audit.theoryClaimGuard.stages[stageName]?.length >= 4);
  }
  assert.ok(audit.theoryClaimGuard.summary.descriptorOnly >= 1);
  assert.ok(audit.theoryClaimGuard.summary.fixtureRecordOnly >= 1);
  assert.ok(audit.theoryClaimGuard.summary.partialPropertyTest >= 1);
  assert.ok(audit.theoryClaimGuard.summary.propertyTestEnforced >= 1);
}
