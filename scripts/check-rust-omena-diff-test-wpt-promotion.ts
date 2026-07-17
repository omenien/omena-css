import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

interface PromotionPolicyV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.wpt-module-promotion-policy";
  readonly template: PromotionTemplateV0;
  readonly modules: readonly PromotionModuleV0[];
}

interface PromotionTemplateV0 {
  readonly minimumFixtureCount: number;
  readonly requiredConsecutiveGreenRuns: number;
  readonly reviewIntervalDays: number;
  readonly knownFailureCeiling: number;
  readonly advisoryHoldReasons: readonly string[];
}

interface PromotionModuleV0 {
  readonly moduleId: string;
  readonly sourceKind: "reviewed-seed" | "extracted-tier-zero";
  readonly stage: "advisory" | "blocking";
  readonly consecutiveGreenRuns: number;
  readonly greenRuns?: readonly {
    readonly date: string;
    readonly commit: string;
    readonly sourcePin: string;
    readonly workflowRunUrl: string;
    readonly conclusion: "success";
  }[];
  readonly reviewedAt: string;
  readonly reviewAfter: string;
  readonly holdReason: string | null;
  readonly evidenceSource: string;
}

interface PromotionTemplateReviewV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.wpt-promotion-template-review";
  readonly policyPath: string;
  readonly reviewedTemplateSha256: string;
  readonly reviewedAt: string;
  readonly decision: string;
}

interface WptManifestV0 {
  readonly source: { readonly pin: string };
  readonly chunks: readonly {
    readonly stage: string;
    readonly fixtureCount: number;
  }[];
  readonly promotion: {
    readonly policy: DerivedArtifactV0;
    readonly templateReview: DerivedArtifactV0;
  };
}

interface DerivedArtifactV0 {
  readonly path: string;
  readonly sha256: string;
  readonly recordCount: number;
}

interface ConformanceDashboardV0 {
  readonly pins: { readonly wptExtraction: string };
  readonly wpt: {
    readonly modules: readonly {
      readonly moduleId: string;
      readonly sourcePin: string;
      readonly evaluatedSubtestCount: number;
      readonly omenaPassCount: number;
      readonly expectedFailureCount: number;
      readonly quarantinedCount: number;
      readonly unexpectedFailureCount: number;
    }[];
  };
}

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const manifest = readJson<WptManifestV0>(path.join(corpusRoot, "manifest.json"));
const policySource = readFileSync(path.join(corpusRoot, manifest.promotion.policy.path), "utf8");
const policy = JSON.parse(policySource) as PromotionPolicyV0;
const templateReviewSource = readFileSync(
  path.join(corpusRoot, manifest.promotion.templateReview.path),
  "utf8",
);
const templateReview = JSON.parse(templateReviewSource) as PromotionTemplateReviewV0;
const dashboard = readJson<ConformanceDashboardV0>(
  path.join(repoRoot, "rust/crates/omena-spec-audit/data/omena-conformance-dashboard.json"),
);
const seedPolicySource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-diff-test/known-failures/wpt-seed-policy.toml"),
  "utf8",
);

assert.equal(policy.schemaVersion, "0");
assert.equal(policy.product, "omena-diff-test.wpt-module-promotion-policy");
assert.equal(sha256(policySource), manifest.promotion.policy.sha256, "promotion policy hash drift");
assert.equal(
  sha256(templateReviewSource),
  manifest.promotion.templateReview.sha256,
  "promotion template review hash drift",
);
assert.equal(policy.modules.length, manifest.promotion.policy.recordCount);
assert.equal(manifest.promotion.templateReview.recordCount, 1);

assert.equal(templateReview.schemaVersion, "0");
assert.equal(templateReview.product, "omena-diff-test.wpt-promotion-template-review");
assert.equal(templateReview.policyPath, manifest.promotion.policy.path);
assert.equal(
  templateReview.reviewedTemplateSha256,
  sha256(JSON.stringify(policy.template)),
  "promotion template changed without reviewed adjudication",
);
assert.ok(templateReview.decision.length > 0);

const legacyMinimum = tomlNumber(seedPolicySource, "required_min_fixture_count_for_stage2");
const legacyGreenRequirement = tomlNumber(seedPolicySource, "required_consecutive_green_runs");
const legacyReviewInterval = tomlNumber(seedPolicySource, "review_interval_days");
assert.equal(policy.template.minimumFixtureCount, legacyMinimum);
assert.equal(policy.template.requiredConsecutiveGreenRuns, legacyGreenRequirement);
assert.equal(policy.template.reviewIntervalDays, legacyReviewInterval);
assert.equal(policy.template.knownFailureCeiling, 0);
assert.ok(policy.template.advisoryHoldReasons.length > 0);
assert.equal(
  new Set(policy.template.advisoryHoldReasons).size,
  policy.template.advisoryHoldReasons.length,
);

const dashboardModuleIds = dashboard.wpt.modules.map((module) => module.moduleId).sort();
const policyModuleIds = policy.modules.map((module) => module.moduleId);
assert.equal(
  new Set(policyModuleIds).size,
  policyModuleIds.length,
  "duplicate promotion module id",
);
assert.deepEqual(
  policyModuleIds.filter((moduleId) => moduleId !== "seed-primary").sort(),
  dashboardModuleIds,
  "promotion policy must cover every extracted WPT module",
);

const today = process.env.OMENA_WPT_PROMOTION_TODAY ?? new Date().toISOString().slice(0, 10);
const seedGreenRunCount = countTomlSections(seedPolicySource, "green_run");
const injectedSeedGreenRunCount = process.env.OMENA_WPT_PROMOTION_TEST_SEED_GREEN_RUNS;
const outcomes = policy.modules.map((module) => {
  assert.ok(existsSync(path.resolve(corpusRoot, module.evidenceSource)), module.evidenceSource);
  assertIsoDate(module.reviewedAt, `${module.moduleId} reviewedAt`);
  assertIsoDate(module.reviewAfter, `${module.moduleId} reviewAfter`);
  assert.equal(
    daysBetween(module.reviewedAt, module.reviewAfter),
    policy.template.reviewIntervalDays,
    `${module.moduleId} review interval drift`,
  );
  if (module.holdReason !== null) {
    assert.ok(
      policy.template.advisoryHoldReasons.includes(module.holdReason),
      `${module.moduleId} has an unreviewed advisory hold reason`,
    );
  }

  const observed =
    module.sourceKind === "reviewed-seed"
      ? seedObservation(manifest, seedPolicySource)
      : extractedObservation(dashboard, module.moduleId);
  const evidencedGreenRuns =
    module.sourceKind === "reviewed-seed" ? seedGreenRunCount : (module.greenRuns ?? []).length;
  if (module.sourceKind === "extracted-tier-zero") {
    const greenRuns = module.greenRuns ?? [];
    assert.equal(
      new Set(greenRuns.map((run) => run.commit)).size,
      greenRuns.length,
      `${module.moduleId} green-run commits must be distinct`,
    );
    for (const run of greenRuns) {
      assertIsoDate(run.date, `${module.moduleId} green run date`);
      assert.match(run.commit, /^[0-9a-f]{40}$/u, `${module.moduleId} green run commit`);
      assert.equal(run.sourcePin, dashboard.pins.wptExtraction);
      assert.match(run.workflowRunUrl, /^https:\/\/github\.com\/.+\/actions\/runs\/[0-9]+$/u);
      assert.equal(run.conclusion, "success");
    }
  }
  assert.equal(
    module.consecutiveGreenRuns,
    evidencedGreenRuns,
    `${module.moduleId} green-run count lacks evidence`,
  );
  const effectiveGreenRuns =
    module.moduleId === "seed-primary" && injectedSeedGreenRunCount !== undefined
      ? Number(injectedSeedGreenRunCount)
      : evidencedGreenRuns;
  assert.ok(Number.isInteger(effectiveGreenRuns) && effectiveGreenRuns >= 0);

  const blockers = promotionBlockers({
    fixtureCount: observed.fixtureCount,
    knownFailureCount: observed.knownFailureCount,
    unexpectedFailureCount: observed.unexpectedFailureCount,
    consecutiveGreenRuns: effectiveGreenRuns,
    reviewAfter: module.reviewAfter,
    today,
    template: policy.template,
  });
  const ready = blockers.length === 0;
  if (ready && module.holdReason === null) {
    assert.equal(module.stage, "blocking", `${module.moduleId} is ready and must be blocking`);
  } else {
    assert.equal(module.stage, "advisory", `${module.moduleId} is not eligible for blocking`);
  }
  return {
    moduleId: module.moduleId,
    stage: module.stage,
    ready,
    fixtureCount: observed.fixtureCount,
    knownFailureCount: observed.knownFailureCount,
    unexpectedFailureCount: observed.unexpectedFailureCount,
    consecutiveGreenRuns: effectiveGreenRuns,
    blockers,
  };
});

const recertified = outcomes.find((outcome) => outcome.moduleId === "seed-primary");
assert.ok(recertified?.ready && recertified.stage === "blocking");
assert.ok(
  outcomes.some((outcome) => outcome.stage === "advisory" && !outcome.ready),
  "new extracted modules must enter through the advisory stage",
);

process.stdout.write(
  `${JSON.stringify({
    product: "omena-diff-test.wpt-module-promotion-gate",
    template: policy.template,
    recertifiedModuleId: recertified.moduleId,
    blockingModuleCount: outcomes.filter((outcome) => outcome.stage === "blocking").length,
    advisoryModuleCount: outcomes.filter((outcome) => outcome.stage === "advisory").length,
    outcomes,
  })}\n`,
);

function seedObservation(
  candidateManifest: WptManifestV0,
  candidatePolicySource: string,
): { fixtureCount: number; knownFailureCount: number; unexpectedFailureCount: number } {
  return {
    fixtureCount: candidateManifest.chunks
      .filter((chunk) => chunk.stage === "stage2-blocking")
      .reduce((count, chunk) => count + chunk.fixtureCount, 0),
    knownFailureCount: countTomlSections(candidatePolicySource, "subtest"),
    unexpectedFailureCount: 0,
  };
}

function extractedObservation(
  candidateDashboard: ConformanceDashboardV0,
  moduleId: string,
): { fixtureCount: number; knownFailureCount: number; unexpectedFailureCount: number } {
  const module = candidateDashboard.wpt.modules.find(
    (candidate) => candidate.moduleId === moduleId,
  );
  assert.ok(module, `${moduleId} is absent from the conformance dashboard`);
  assert.equal(module.sourcePin, candidateDashboard.pins.wptExtraction);
  assert.ok(module.unexpectedFailureCount >= 0, `${moduleId} failure accounting underflow`);
  return {
    fixtureCount: module.evaluatedSubtestCount,
    knownFailureCount: module.expectedFailureCount + module.quarantinedCount,
    unexpectedFailureCount: module.unexpectedFailureCount,
  };
}

function promotionBlockers(input: {
  readonly fixtureCount: number;
  readonly knownFailureCount: number;
  readonly unexpectedFailureCount: number;
  readonly consecutiveGreenRuns: number;
  readonly reviewAfter: string;
  readonly today: string;
  readonly template: PromotionTemplateV0;
}): string[] {
  const blockers: string[] = [];
  if (input.fixtureCount < input.template.minimumFixtureCount) blockers.push("fixture-floor");
  if (input.knownFailureCount > input.template.knownFailureCeiling) {
    blockers.push("known-failure-ceiling");
  }
  if (input.unexpectedFailureCount > 0) blockers.push("unexpected-failures");
  if (input.consecutiveGreenRuns < input.template.requiredConsecutiveGreenRuns) {
    blockers.push("consecutive-green-runs");
  }
  if (input.today > input.reviewAfter) blockers.push("review-expired");
  return blockers;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function tomlNumber(source: string, key: string): number {
  const match = new RegExp(`^${key}\\s*=\\s*([0-9]+)$`, "mu").exec(source);
  assert.ok(match, `${key} is absent from the seed policy`);
  return Number(match[1]);
}

function countTomlSections(source: string, section: string): number {
  return [...source.matchAll(new RegExp(`^\\[\\[${section}\\]\\]$`, "gmu"))].length;
}

function assertIsoDate(value: string, label: string): void {
  assert.match(value, /^\d{4}-\d{2}-\d{2}$/u, `${label} must be YYYY-MM-DD`);
}

function daysBetween(start: string, end: string): number {
  return (Date.parse(`${end}T00:00:00Z`) - Date.parse(`${start}T00:00:00Z`)) / 86_400_000;
}
