import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";

import { formatGeneratedJson } from "./generated-json";

const DASHBOARD_PATH = "rust/crates/omena-spec-audit/data/omena-conformance-dashboard.json";
const COVERAGE_GAP_PATH = "rust/crates/omena-spec-audit/data/omena-coverage-gap.json";
const SPEC_SOURCES_PATH = "rust/crates/omena-spec-audit/data/spec-sources.json";
const WPT_MANIFEST_PATH = "rust/crates/omena-diff-test/wpt-corpus/manifest.json";

interface WptReportV0 {
  readonly product: string;
  readonly extractedCorpusMode: string;
  readonly extractedSourcePin: string;
  readonly extractedTupleCount: number;
  readonly extractedEvaluatedTupleCount: number;
  readonly extractedOutcomeCube: Readonly<Record<string, number>>;
  readonly extractedModuleOutcomes: readonly WptModuleOutcomeV0[];
  readonly extractedCases: readonly WptCaseOutcomeV0[];
}

interface WptModuleOutcomeV0 {
  readonly moduleId: string;
  readonly sourcePin: string;
  readonly extractedSubtestCount: number;
  readonly evaluatedSubtestCount: number;
  readonly omenaPassCount: number;
  readonly lightningPassCount: number;
  readonly expectedSetWitnessCount: number;
  readonly expectedFailureCount: number;
  readonly quarantinedCount: number;
  readonly unexpectedFailureCount: number;
  readonly skippedDynamicCallCount: number;
  readonly nonTierZeroFileCount: number;
}

interface WptCaseOutcomeV0 {
  readonly id: string;
  readonly moduleId: string;
  readonly wptPath: string;
  readonly specLinks: readonly string[];
  readonly omenaPass: boolean;
  readonly lightningPass: boolean;
  readonly wptExpectedPass: boolean;
  readonly expectation?: {
    readonly status: "expected-failure" | "quarantined";
    readonly reasonCode: string;
  };
}

interface CoverageGapV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly policy: {
    readonly denominator: string;
    readonly capabilityTiers: readonly string[];
    readonly namedReasons: readonly string[];
  };
  readonly summary: {
    readonly categoryCounts: Readonly<Record<string, number>>;
    readonly tierCounts: Readonly<Record<string, number>>;
    readonly categoryTierCounts: Readonly<Record<string, Readonly<Record<string, number>>>>;
    readonly namedReasonCounts: Readonly<Record<string, number>>;
    readonly rowCount: number;
    readonly unassignedCount: number;
  };
}

interface RuntimeLedgerV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly recordCount: number;
  readonly modelConformantCount: number;
  readonly divergentCount: number;
  readonly notExercisedCount: number;
  readonly measuredComparisonCount: number;
  readonly aggregatePolicy: string;
  readonly complete: boolean;
}

interface SpecSourcesV0 {
  readonly sources: readonly {
    readonly name: string;
    readonly package: string;
    readonly version: string;
    readonly gitHead?: string;
    readonly repoPin?: string;
    readonly modulePath?: string;
    readonly role: string;
  }[];
}

interface WptManifestV0 {
  readonly extraction: {
    readonly sourcePin: string;
    readonly tuples: { readonly sha256: string; readonly recordCount: number };
    readonly coverage: { readonly sha256: string };
    readonly moduleCoverage: readonly { readonly moduleId: string; readonly wptPath: string }[];
  };
  readonly expectations: {
    readonly reviewPolicy: { readonly sha256: string };
  };
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");

async function main(): Promise<void> {
  const wpt = runJson<WptReportV0>(
    "full WPT conformance",
    "node",
    ["--import", "tsx", "./scripts/check-rust-omena-diff-test-wpt-seed.ts"],
    {
      OMENA_WPT_FULL_CORPUS: "1",
      OMENA_WPT_INCLUDE_CASES: "1",
    },
  );
  const runtimeLedger = runJson<RuntimeLedgerV0>("transform conformance ledger", "node", [
    "--import",
    "tsx",
    "./scripts/check-rust-omena-diff-test-transform-pass-cascade-conformance.ts",
  ]);
  const coverageSource = readFileSync(path.join(repoRoot, COVERAGE_GAP_PATH), "utf8");
  const coverage = JSON.parse(coverageSource) as CoverageGapV0;
  const specSources = readJson<SpecSourcesV0>(SPEC_SOURCES_PATH);
  const wptManifest = readJson<WptManifestV0>(WPT_MANIFEST_PATH);

  assert.equal(wpt.product, "omena-diff-test.wpt-seed-three-way-report");
  assert.equal(wpt.extractedCorpusMode, "full");
  assert.equal(wpt.extractedTupleCount, wpt.extractedEvaluatedTupleCount);
  assert.equal(wpt.extractedCases.length, wpt.extractedTupleCount);
  assert.equal(wpt.extractedSourcePin, wptManifest.extraction.sourcePin);
  assert.ok(wpt.extractedCases.every((entry) => entry.wptExpectedPass));
  assert.equal(coverage.product, "omena-spec-audit.coverage-gap");
  assert.equal(runtimeLedger.product, "omena-diff-test.transform-pass-cascade-conformance");
  assert.equal(runtimeLedger.complete, true);

  const webref = specSources.sources.find((source) => source.package === "@webref/css");
  assert.ok(webref?.gitHead, "dashboard requires a pinned Webref source");
  const webrefPin = `@webref/css@${webref.version}#${webref.gitHead}`;
  const modulePinById = new Map<string, string>();
  for (const module of wptManifest.extraction.moduleCoverage) {
    const source = specSources.sources.find(
      (candidate) =>
        candidate.role === "external-wpt-corpus-module" && candidate.modulePath === module.wptPath,
    );
    assert.ok(source?.repoPin, `${module.moduleId} source pin is missing`);
    assert.equal(source.repoPin, wpt.extractedSourcePin, `${module.moduleId} source pin drift`);
    modulePinById.set(module.moduleId, source.repoPin);
  }

  const specBuckets = new Map<string, WptCaseOutcomeV0[]>();
  for (const entry of wpt.extractedCases) {
    const links = [...new Set(entry.specLinks.map(normalizeSpecLink))];
    const bucketKeys = links.length > 0 ? links : [`unlinked:${entry.moduleId}`];
    for (const key of bucketKeys) {
      const bucket = specBuckets.get(key) ?? [];
      bucket.push(entry);
      specBuckets.set(key, bucket);
    }
  }
  const referenceResults = [...specBuckets.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([spec, cases]) => {
      const moduleIds = [...new Set(cases.map((entry) => entry.moduleId))].sort();
      return {
        spec,
        pins: {
          wpt: moduleIds.map((moduleId) => {
            const sourcePin = modulePinById.get(moduleId);
            assert.ok(sourcePin, `${moduleId} lacks a dashboard pin`);
            return { moduleId, sourcePin };
          }),
          webref: webrefPin,
        },
        evaluatedCaseCount: cases.length,
        omenaConformantCaseCount: cases.filter((entry) => entry.omenaPass).length,
        lightningConformantCaseCount: cases.filter((entry) => entry.lightningPass).length,
        expectedFailureCount: cases.filter(
          (entry) => entry.expectation?.status === "expected-failure",
        ).length,
        quarantinedCount: cases.filter((entry) => entry.expectation?.status === "quarantined")
          .length,
      };
    });
  const specResults = referenceResults.filter(
    (entry) => classifyReference(entry.spec) === "css-spec",
  );
  const relatedReferenceResults = referenceResults.filter(
    (entry) => classifyReference(entry.spec) === "related-reference",
  );
  const unlinkedResults = referenceResults.filter(
    (entry) => classifyReference(entry.spec) === "unlinked",
  );
  assert.equal(
    wpt.extractedModuleOutcomes.reduce((count, module) => count + module.evaluatedSubtestCount, 0),
    wpt.extractedEvaluatedTupleCount,
  );
  assert.equal(
    Object.values(wpt.extractedOutcomeCube).reduce((count, value) => count + value, 0),
    wpt.extractedEvaluatedTupleCount,
  );

  const report = {
    schemaVersion: "0",
    product: "omena-spec-audit.conformance-dashboard",
    policy: {
      headlinePercentageAllowed: false,
      denominator: coverage.policy.denominator,
      wptCaseAccounting:
        "Every extracted tier-zero subtest is evaluated once; a tuple linked to multiple specs contributes once to each linked spec bucket.",
      unlinkedCaseAccounting:
        "Cases without rel=help links remain visible in a per-module unlinked bucket.",
      runtimeCounts:
        "Regenerated by executing the current conformance runners; never hand-authored.",
    },
    pins: {
      wptExtraction: wpt.extractedSourcePin,
      webref: webrefPin,
    },
    inputFingerprints: {
      wptTuplesSha256: wptManifest.extraction.tuples.sha256,
      wptCoverageSha256: wptManifest.extraction.coverage.sha256,
      expectationReviewSha256: wptManifest.expectations.reviewPolicy.sha256,
      coverageGapSha256: createHash("sha256").update(coverageSource).digest("hex"),
    },
    wpt: {
      evaluatedCaseCount: wpt.extractedEvaluatedTupleCount,
      outcomeCube: wpt.extractedOutcomeCube,
      modules: wpt.extractedModuleOutcomes,
      specs: specResults,
      relatedReferences: relatedReferenceResults,
      unlinkedModules: unlinkedResults,
    },
    capabilityLedger: {
      rowCount: coverage.summary.rowCount,
      categoryCounts: coverage.summary.categoryCounts,
      tierCounts: coverage.summary.tierCounts,
      categoryTierCounts: coverage.summary.categoryTierCounts,
      unassignedCount: coverage.summary.unassignedCount,
      namedReasonCounts: coverage.summary.namedReasonCounts,
      capabilityTiers: coverage.policy.capabilityTiers,
      namedReasons: coverage.policy.namedReasons,
    },
    runtimeConformanceLedger: runtimeLedger,
  };
  const source = await formatGeneratedJson(DASHBOARD_PATH, report);
  const dashboardPath = path.join(repoRoot, DASHBOARD_PATH);
  if (checkOnly) {
    assert.equal(
      readFileSync(dashboardPath, "utf8"),
      source,
      `${DASHBOARD_PATH} is stale; run the dashboard update command`,
    );
  } else {
    writeFileSync(dashboardPath, source);
  }

  process.stdout.write(
    `${JSON.stringify({
      product: "omena-spec-audit.conformance-dashboard-generator",
      mode: checkOnly ? "check" : "write",
      evaluatedCaseCount: wpt.extractedEvaluatedTupleCount,
      moduleCount: wpt.extractedModuleOutcomes.length,
      specBucketCount: specResults.length,
      relatedReferenceBucketCount: relatedReferenceResults.length,
      unlinkedBucketCount: unlinkedResults.length,
      capabilityLedgerRowCount: coverage.summary.rowCount,
      runtimeLedgerRecordCount: runtimeLedger.recordCount,
      pins: report.pins,
    })}\n`,
  );
}

function runJson<T>(
  label: string,
  command: string,
  args: readonly string[],
  environment: Readonly<Record<string, string>> = {},
): T {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, ...environment },
    maxBuffer: 128 * 1024 * 1024,
  });
  assert.equal(
    result.status,
    0,
    `${label} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as T;
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8")) as T;
}

function normalizeSpecLink(link: string): string {
  const url = new URL(link);
  url.hash = "";
  url.search = "";
  return url.toString();
}

function classifyReference(reference: string): "css-spec" | "related-reference" | "unlinked" {
  if (reference.startsWith("unlinked:")) return "unlinked";
  const url = new URL(reference);
  if (url.hostname === "drafts.csswg.org") return "css-spec";
  if (url.hostname === "www.w3.org" && url.pathname.startsWith("/TR/css-")) return "css-spec";
  return "related-reference";
}

void main();
