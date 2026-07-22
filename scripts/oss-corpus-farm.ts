import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { runCheckerCli } from "../server/checker-cli/src";

type Dialect = "css" | "scss" | "less" | "sass";
type ExpectationKind =
  | "static-must-match"
  | "expected-sound-bail"
  | "parser-recovery"
  | "out-of-scope"
  | "finding-census";
type Stage = "stage1-advisory" | "stage2-blocking";
type DiffKind = "pass" | "missing-baseline" | "pin-change" | "regression";

interface ExternalCorpusDifferentialManifestV1 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly selectionCriteria: OssCorpusFarmSelectionCriteriaV0;
  readonly lintCensus: RealWorkspaceLintCensusManifestV0;
  readonly fixtures: readonly ExternalCorpusEnvelopeV1[];
}

interface RealWorkspaceLintCensusManifestV0 {
  readonly policyPath: string;
  readonly coveragePath: string;
  readonly reportPath: string;
  readonly fixedTimeBudgetSeconds: number;
  readonly perSourceFileBudgetSeconds: number;
}

interface OssCorpusFarmSelectionCriteriaV0 {
  readonly minimumDialectCount: number;
  readonly maxSparsePathCountPerEntry: number;
  readonly maxChunkSourceBytes: number;
  readonly maxSelectedWorktreeFiles: number;
  readonly maxSelectedWorktreeBytes: number;
}

interface ExternalCorpusEnvelopeV1 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: Stage;
  readonly dialect?: Dialect;
  readonly expectationKind?: ExpectationKind;
  readonly source: PinnedRepositoryCorpusSourceV1 | LocalWorkspaceCorpusSourceV1;
  readonly generation: {
    readonly tool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs?: readonly string[];
  };
  readonly provenance?: {
    readonly generationTool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs: readonly string[];
  };
  readonly chunks: readonly {
    readonly chunkId: string;
    readonly path: string;
    readonly sha256: string;
    readonly fixtureCount: number;
  }[];
}

interface PinnedRepositoryCorpusSourceV1 {
  readonly kind: "pinned-repository";
  readonly repository: string;
  readonly pin: string;
  readonly sparsePaths: readonly string[];
  readonly helperClasses: readonly string[];
  readonly layoutDependentHelpersExcluded: readonly string[];
}

interface LocalWorkspaceCorpusSourceV1 {
  readonly kind: "local-workspace";
  readonly workspacePath: string;
}

interface LocalWorkspaceSelectionV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly id: string;
  readonly sourceKind: "local-workspace";
  readonly workspacePath: string;
  readonly includedExtensions: readonly string[];
  readonly selectedFileCount: number;
}

interface RealWorkspaceLintCensusPolicyV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly generatedBy: string;
  readonly entries: readonly RealWorkspaceLintCensusPolicyEntryV0[];
  readonly knownFalsePositiveClasses: readonly KnownFalsePositiveClassV0[];
  readonly zeroBudgetClasses: readonly ZeroBudgetClassV0[];
}

interface RealWorkspaceLintCensusPolicyEntryV0 {
  readonly id: string;
  readonly truePositivePins: readonly FindingPinV0[];
  readonly knownFalsePositivePins: readonly KnownFalsePositivePinV0[];
}

interface FindingPinV0 {
  readonly ruleId: string;
  readonly filePath: string;
  readonly range: LintRangeV0;
  readonly messageSha256: string;
}

interface KnownFalsePositivePinV0 extends FindingPinV0 {
  readonly causeClass: string;
}

interface KnownFalsePositiveClassV0 {
  readonly ruleId: string;
  readonly causeClass: string;
  readonly baselineCount: number;
  readonly firstSeen: string;
  readonly owner: string;
  readonly adjudicationNote: string;
}

interface ZeroBudgetClassV0 {
  readonly ruleId: string;
  readonly causeClass: string;
  readonly ownerCommits: readonly string[];
  readonly matcher:
    | { readonly kind: "declaration-pairs"; readonly values: readonly string[] }
    | { readonly kind: "specifier-prefixes"; readonly prefixes: readonly string[] };
}

interface LintRangeV0 {
  readonly start: { readonly line: number; readonly character: number };
  readonly end: { readonly line: number; readonly character: number };
}

interface ProductLintFindingV0 {
  readonly filePath: string;
  readonly ruleId: string;
  readonly severity: string;
  readonly range: LintRangeV0;
  readonly message: string;
}

interface ProductLintReportV0 {
  readonly styleFileCount: number;
  readonly sourceFileCount: number;
  readonly findingCount: number;
  readonly tiers: readonly { readonly findings: readonly ProductLintFindingV0[] }[];
}

interface ProductLintEnvelopeV0 {
  readonly product: string;
  readonly payload: ProductLintReportV0;
}

interface NormalizedLintFindingV0 extends FindingPinV0 {
  readonly severity: string;
}

interface RealWorkspaceLintCensusReportV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly generatedBy: string;
  readonly coverageDigest: string;
  readonly policyDigest: string;
  readonly entryCount: number;
  readonly totalFindingCount: number;
  readonly truePositiveCount: number;
  readonly knownFalsePositiveCount: number;
  readonly unclassifiedCount: number;
  readonly zeroBudgetViolationCount: number;
  readonly entries: readonly RealWorkspaceLintCensusEntryReportV0[];
}

interface RealWorkspaceLintCensusEntryReportV0 {
  readonly id: string;
  readonly corpusDigest: string;
  readonly styleFileCount: number;
  readonly sourceFileCount: number;
  readonly findingCount: number;
  readonly truePositiveCount: number;
  readonly knownFalsePositiveCount: number;
  readonly unclassifiedCount: number;
  readonly zeroBudgetViolationCount: number;
  readonly findingCountsByRule: Readonly<Record<string, number>>;
  readonly knownFalsePositiveCountsByClass: Readonly<Record<string, number>>;
}

interface BaselineLedgerV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly generatedBy: string;
  readonly baselines: readonly BaselineRecordV0[];
}

interface BaselineRecordV0 {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly factSetHash: string;
  readonly factCount: number;
}

interface CheckerReportV1 {
  readonly sourceFiles?: readonly string[];
  readonly styleFiles?: readonly string[];
  readonly summary?: {
    readonly warnings?: number;
    readonly hints?: number;
    readonly total?: number;
  };
  readonly findings?: readonly {
    readonly code?: string;
    readonly severity?: string;
    readonly message?: string;
    readonly filePath?: string;
    readonly range?: unknown;
  }[];
}

interface FactSetRecordV0 extends BaselineRecordV0 {
  readonly canonicalJson: string;
}

interface FarmReportV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly entryCount: number;
  readonly comparedCount: number;
  readonly passCount: number;
  readonly pinChangeCount: number;
  readonly regressionCount: number;
  readonly missingBaselineCount: number;
  readonly reports: readonly FarmEntryReportV0[];
}

interface FarmEntryReportV0 {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly factSetHash: string;
  readonly factCount: number;
  readonly diffKind: DiffKind;
}

interface RegressionManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly fixtures: readonly RegressionManifestFixtureV0[];
}

interface RegressionManifestFixtureV0 {
  readonly id: string;
  readonly path: string;
  readonly status: string;
  readonly issue?: {
    readonly repository: string;
    readonly number: number;
  };
  readonly sourceProvenance?: {
    readonly repository: string;
    readonly pin: string;
  };
  readonly minimization?: string;
}

const repoRoot = process.cwd();
const farmRoot = path.join(repoRoot, "rust/crates/omena-diff-test/oss-corpus-farm");
const manifestPath = path.join(farmRoot, "manifest.json");
const baselinePath = path.join(farmRoot, "baselines.json");
const reportPath = path.join(farmRoot, "report.json");
const regressionRoot = path.join(repoRoot, "rust/crates/omena-diff-test/regressions");
const regressionManifestPath = path.join(regressionRoot, "manifest.json");
const rawCaptureRoot = process.env.OMENA_OSS_CORPUS_CAPTURE_DIR
  ? path.resolve(repoRoot, process.env.OMENA_OSS_CORPUS_CAPTURE_DIR)
  : regressionRoot;

const args = new Set(process.argv.slice(2));

void (async () => {
  if (args.has("--classifier-fixture")) {
    process.stdout.write(`${JSON.stringify(classifierFixtureReport())}\n`);
    return;
  }
  if (args.has("--path-policy-fixture")) {
    const candidates = ["src", "src/styles", ".", "", "../outside", "/absolute"];
    process.stdout.write(
      `${JSON.stringify(candidates.map((candidate) => [candidate, isBoundedPath(candidate)]))}\n`,
    );
    return;
  }

  if (args.has("--determinism-fixture")) {
    const fixturePath = valueAfter("--determinism-fixture");
    await checkDeterministicProjection(path.resolve(repoRoot, fixturePath));
    return;
  }

  const manifest = readManifest();
  assertManifest(manifest);

  if (args.has("--lint-census") || args.has("--write-lint-census")) {
    const report = runRealWorkspaceLintCensus(manifest);
    const reportPathForManifest = resolveFarmPath(manifest.lintCensus.reportPath);
    const rendered = `${JSON.stringify(report, null, 2)}\n`;
    if (args.has("--write-lint-census")) {
      writeFileSync(reportPathForManifest, rendered);
    } else {
      assert.ok(existsSync(reportPathForManifest), "lint census report must be committed");
      assert.equal(
        readFileSync(reportPathForManifest, "utf8"),
        rendered,
        "real-workspace lint census report drifted; regenerate it with --write-lint-census",
      );
    }
    process.stdout.write(rendered);
    return;
  }

  if (args.has("--write-baseline")) {
    const fresh = await runFarm(manifest.fixtures);
    writeBaselines(fresh);
    writeReport(buildReport(fresh, fresh));
    return;
  }

  const baselines = readJson<BaselineLedgerV0>(baselinePath);
  assert.equal(baselines.schemaVersion, "0");
  assert.equal(baselines.product, "omena-diff-test.oss-corpus-farm.baselines");
  const fresh = await runFarm(manifest.fixtures);
  const report = buildReport(fresh, baselines.baselines);
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);

  const failures = report.reports.filter((entry) => entry.diffKind !== "pass");
  assert.deepEqual(
    failures,
    [],
    `oss corpus farm detected baseline differences:\n${JSON.stringify(failures, null, 2)}`,
  );
})();

async function checkDeterministicProjection(workspaceRoot: string): Promise<void> {
  assert.ok(existsSync(workspaceRoot), `determinism fixture must exist: ${workspaceRoot}`);
  const left = await projectWorkspaceFactSet({
    id: "local-determinism-fixture",
    repository: "local",
    pin: "local@0000000000000000000000000000000000000000",
    checkoutDir: workspaceRoot,
  });
  const right = await projectWorkspaceFactSet({
    id: "local-determinism-fixture",
    repository: "local",
    pin: "local@0000000000000000000000000000000000000000",
    checkoutDir: workspaceRoot,
  });
  assert.equal(left.factSetHash, right.factSetHash);
  assert.ok(left.factCount > 0, "deterministic fixture must produce at least one fact");
  process.stdout.write(
    `${JSON.stringify(
      {
        product: "omena-diff-test.oss-corpus-farm.determinism",
        factSetHash: left.factSetHash,
        factCount: left.factCount,
      },
      null,
      2,
    )}\n`,
  );
}

function runRealWorkspaceLintCensus(
  manifest: ExternalCorpusDifferentialManifestV1,
): RealWorkspaceLintCensusReportV0 {
  assertWorkspaceLintScanShape();
  const policyPath = resolveFarmPath(manifest.lintCensus.policyPath);
  const coveragePath = resolveFarmPath(manifest.lintCensus.coveragePath);
  const policy = readJson<RealWorkspaceLintCensusPolicyV0>(policyPath);
  assertRealWorkspaceLintCensusPolicy(policy, manifest);
  assertRealWorkspaceLintCensusCoverage(coveragePath, manifest);

  const policyEntries = policy.entries.map((entry) => ({
    ...entry,
    truePositivePins: [...entry.truePositivePins],
    knownFalsePositivePins: [...entry.knownFalsePositivePins],
  }));
  const knownFalsePositiveClasses = policy.knownFalsePositiveClasses.map((entry) => ({ ...entry }));
  if (args.has("--inject-lint-census-tp-pin-removal")) {
    const first = policyEntries.find((entry) => entry.truePositivePins.length > 0);
    assert.ok(first, "fault injection requires a true-positive pin");
    first.truePositivePins = first.truePositivePins.slice(1);
  }
  if (args.has("--inject-lint-census-ratchet-increase")) {
    const first = knownFalsePositiveClasses[0];
    assert.ok(first, "fault injection requires a known false-positive class");
    first.baselineCount += 1;
  }

  const effectivePolicy: RealWorkspaceLintCensusPolicyV0 = {
    ...policy,
    entries: policyEntries,
    knownFalsePositiveClasses,
  };
  assertRealWorkspaceLintCensusPolicy(effectivePolicy, manifest);

  const omenaBinary = resolveOmenaBinary();
  const requestedEntryId = optionalValueAfter("--lint-census-entry");
  const localEntries = manifest.fixtures
    .filter(isLocalLintCensusEntry)
    .filter((entry) => !requestedEntryId || entryId(entry) === requestedEntryId);
  if (requestedEntryId) {
    assert.equal(
      localEntries.length,
      1,
      `requested lint census entry must exist: ${requestedEntryId}`,
    );
  }
  const entryReports = localEntries.map((entry) => {
    const policyEntry = effectivePolicy.entries.find(
      (candidate) => candidate.id === entryId(entry),
    );
    assert.ok(policyEntry, `${entryId(entry)} must have a lint census policy entry`);
    return runRealWorkspaceLintCensusEntry(
      entry,
      policyEntry,
      effectivePolicy.zeroBudgetClasses,
      manifest.lintCensus,
      omenaBinary,
    );
  });

  const totalFindingCount = sumBy(entryReports, (entry) => entry.findingCount);
  const truePositiveCount = sumBy(entryReports, (entry) => entry.truePositiveCount);
  const knownFalsePositiveCount = sumBy(entryReports, (entry) => entry.knownFalsePositiveCount);
  const unclassifiedCount = sumBy(entryReports, (entry) => entry.unclassifiedCount);
  const zeroBudgetViolationCount = sumBy(entryReports, (entry) => entry.zeroBudgetViolationCount);
  assert.equal(
    truePositiveCount + knownFalsePositiveCount,
    totalFindingCount,
    "every product finding must have exactly one reviewed classification",
  );
  assert.equal(unclassifiedCount, 0, "real-workspace lint census found unclassified findings");
  assert.equal(
    zeroBudgetViolationCount,
    0,
    "real-workspace lint census found a zero-budget regression",
  );

  const report: RealWorkspaceLintCensusReportV0 = {
    schemaVersion: "0",
    product: "omena-diff-test.real-workspace-lint-census",
    generatedBy: "scripts/oss-corpus-farm.ts",
    coverageDigest: sha256(readFileSync(coveragePath)),
    policyDigest: sha256(stableStringify(effectivePolicy)),
    entryCount: entryReports.length,
    totalFindingCount,
    truePositiveCount,
    knownFalsePositiveCount,
    unclassifiedCount,
    zeroBudgetViolationCount,
    entries: entryReports,
  };
  assertHonestCensusVocabulary(stableStringify(report));
  return report;
}

function runRealWorkspaceLintCensusEntry(
  entry: ExternalCorpusEnvelopeV1 & { readonly source: LocalWorkspaceCorpusSourceV1 },
  policyEntry: RealWorkspaceLintCensusPolicyEntryV0,
  zeroBudgetClasses: readonly ZeroBudgetClassV0[],
  budget: RealWorkspaceLintCensusManifestV0,
  omenaBinary: string,
): RealWorkspaceLintCensusEntryReportV0 {
  const id = entryId(entry);
  const workspaceRoot = path.resolve(repoRoot, entry.source.workspacePath);
  assert.ok(existsSync(workspaceRoot), `${id} local workspace must exist`);
  const startedAt = Date.now();
  const result = run(omenaBinary, ["lint", workspaceRoot, "--profile", "recommended", "--json"]);
  const elapsedSeconds = (Date.now() - startedAt) / 1000;
  const envelope = JSON.parse(result.stdout) as ProductLintEnvelopeV0;
  assert.equal(envelope.product, "omena-cli.lint", `${id} must use the product lint envelope`);
  const productReport = envelope.payload;
  let productFindings = productReport.tiers.flatMap((tier) => tier.findings);
  if (args.has("--inject-lint-census-finding-drop") && productFindings.length > 0) {
    productFindings = productFindings.slice(1);
  }
  assert.equal(
    productFindings.length,
    productReport.findingCount,
    `${id} flattened findings must equal the product report total`,
  );
  const elapsedBudgetSeconds =
    budget.fixedTimeBudgetSeconds +
    budget.perSourceFileBudgetSeconds * productReport.sourceFileCount;
  assert.ok(
    elapsedSeconds <= elapsedBudgetSeconds,
    `${id} lint took ${elapsedSeconds.toFixed(2)}s, exceeding the ${elapsedBudgetSeconds.toFixed(2)}s linear budget`,
  );

  const findings = productFindings.map((finding) => normalizeLintFinding(workspaceRoot, finding));
  const findingKeys = findings.map(findingPinKey);
  assert.equal(new Set(findingKeys).size, findingKeys.length, `${id} findings must be unique`);
  const truePositiveKeys = new Set(policyEntry.truePositivePins.map(findingPinKey));
  const knownFalsePositiveKeys = new Set(policyEntry.knownFalsePositivePins.map(findingPinKey));
  for (const pin of policyEntry.truePositivePins) {
    assert.ok(
      findingKeys.includes(findingPinKey(pin)),
      `${id} true-positive pin disappeared: ${findingPinKey(pin)}`,
    );
  }
  for (const pin of policyEntry.knownFalsePositivePins) {
    assert.ok(
      findingKeys.includes(findingPinKey(pin)),
      `${id} known false-positive pin changed without ledger review: ${findingPinKey(pin)}`,
    );
  }

  let truePositiveCount = 0;
  let knownFalsePositiveCount = 0;
  let unclassifiedCount = 0;
  let zeroBudgetViolationCount = 0;
  const knownFalsePositiveCountsByClass = new Map<string, number>();
  for (const finding of findings) {
    const key = findingPinKey(finding);
    const truePositive = truePositiveKeys.has(key);
    const knownFalsePositive = knownFalsePositiveKeys.has(key);
    assert.ok(
      !(truePositive && knownFalsePositive),
      `${id} finding cannot be both true-positive and known false-positive: ${key}`,
    );
    const zeroBudgetClass = zeroBudgetClasses.find((candidate) =>
      findingMatchesZeroBudgetClass(workspaceRoot, finding, candidate),
    );
    if (zeroBudgetClass) {
      zeroBudgetViolationCount += 1;
      process.stderr.write(
        `${id}: zero-budget ${zeroBudgetClass.ruleId}/${zeroBudgetClass.causeClass}: ${finding.filePath}\n`,
      );
      continue;
    }
    if (truePositive) {
      truePositiveCount += 1;
      continue;
    }
    if (knownFalsePositive) {
      knownFalsePositiveCount += 1;
      const pin = policyEntry.knownFalsePositivePins.find(
        (candidate) => findingPinKey(candidate) === key,
      );
      assert.ok(pin, `${id} known false-positive finding must have a reviewed pin`);
      const classKey = `${pin.ruleId}/${pin.causeClass}`;
      knownFalsePositiveCountsByClass.set(
        classKey,
        (knownFalsePositiveCountsByClass.get(classKey) ?? 0) + 1,
      );
      continue;
    }
    unclassifiedCount += 1;
    process.stderr.write(`${id}: unclassified ${key}\n`);
  }

  const findingCountsByRule = Object.fromEntries(
    [...new Set(findings.map((finding) => finding.ruleId))]
      .sort((left, right) => left.localeCompare(right, "en"))
      .map((ruleId) => [ruleId, findings.filter((finding) => finding.ruleId === ruleId).length]),
  );
  process.stderr.write(
    `${id}: ${productReport.findingCount} findings over ${productReport.sourceFileCount} source files in ${elapsedSeconds.toFixed(2)}s (budget ${elapsedBudgetSeconds.toFixed(2)}s)\n`,
  );
  return {
    id,
    corpusDigest: workspaceContentDigest(workspaceRoot),
    styleFileCount: productReport.styleFileCount,
    sourceFileCount: productReport.sourceFileCount,
    findingCount: productReport.findingCount,
    truePositiveCount,
    knownFalsePositiveCount,
    unclassifiedCount,
    zeroBudgetViolationCount,
    findingCountsByRule,
    knownFalsePositiveCountsByClass: Object.fromEntries(
      [...knownFalsePositiveCountsByClass.entries()].sort(([left], [right]) =>
        left.localeCompare(right, "en"),
      ),
    ),
  };
}

function assertRealWorkspaceLintCensusPolicy(
  policy: RealWorkspaceLintCensusPolicyV0,
  manifest: ExternalCorpusDifferentialManifestV1,
): void {
  assert.equal(policy.schemaVersion, "0");
  assert.equal(policy.product, "omena-diff-test.real-workspace-lint-census-policy");
  assert.equal(policy.generatedBy, "reviewed finding adjudication");
  assertHonestCensusVocabulary(stableStringify(policy));
  const localEntryIds = manifest.fixtures
    .filter(isLocalLintCensusEntry)
    .map(entryId)
    .sort((left, right) => left.localeCompare(right, "en"));
  const policyEntryIds = policy.entries
    .map((entry) => entry.id)
    .sort((left, right) => left.localeCompare(right, "en"));
  assert.deepEqual(
    policyEntryIds,
    localEntryIds,
    "lint census policy must cover every local entry",
  );
  assert.ok(localEntryIds.length >= 2, "lint census requires at least two local workspaces");

  const allTruePositiveKeys = policy.entries.flatMap((entry) =>
    entry.truePositivePins.map((pin) => `${entry.id}:${findingPinKey(pin)}`),
  );
  const allKnownFalsePositiveKeys = policy.entries.flatMap((entry) =>
    entry.knownFalsePositivePins.map((pin) => `${entry.id}:${findingPinKey(pin)}`),
  );
  assert.equal(
    new Set(allTruePositiveKeys).size,
    allTruePositiveKeys.length,
    "true-positive pins must be unique",
  );
  assert.equal(
    new Set(allKnownFalsePositiveKeys).size,
    allKnownFalsePositiveKeys.length,
    "known false-positive pins must be unique",
  );
  assert.deepEqual(
    allTruePositiveKeys.filter((key) => new Set(allKnownFalsePositiveKeys).has(key)),
    [],
    "classification buckets must be disjoint",
  );
  for (const entry of policy.entries) {
    assert.ok(entry.truePositivePins.length > 0, `${entry.id} must pin a genuine finding`);
  }

  const knownClassKeys = policy.knownFalsePositiveClasses.map(
    (entry) => `${entry.ruleId}/${entry.causeClass}`,
  );
  assert.equal(
    new Set(knownClassKeys).size,
    knownClassKeys.length,
    "known false-positive class rows must be unique",
  );
  for (const knownClass of policy.knownFalsePositiveClasses) {
    assert.ok(knownClass.adjudicationNote.trim().length > 0);
    assert.ok(knownClass.owner.trim().length > 0);
    const pins = policy.entries.flatMap((entry) =>
      entry.knownFalsePositivePins.filter(
        (pin) => pin.ruleId === knownClass.ruleId && pin.causeClass === knownClass.causeClass,
      ),
    );
    assert.equal(
      knownClass.baselineCount,
      pins.length,
      `${knownClass.ruleId}/${knownClass.causeClass} baseline must equal its reviewed pins`,
    );
  }
  for (const pin of policy.entries.flatMap((entry) => entry.knownFalsePositivePins)) {
    assert.ok(
      policy.knownFalsePositiveClasses.some(
        (candidate) => candidate.ruleId === pin.ruleId && candidate.causeClass === pin.causeClass,
      ),
      `known false-positive pin lacks a class ledger row: ${pin.ruleId}/${pin.causeClass}`,
    );
  }
  const zeroClassKeys = policy.zeroBudgetClasses.map(
    (entry) => `${entry.ruleId}/${entry.causeClass}`,
  );
  assert.equal(
    new Set(zeroClassKeys).size,
    zeroClassKeys.length,
    "zero-budget class rows must be unique",
  );
  assert.deepEqual(
    zeroClassKeys.filter((key) => new Set(knownClassKeys).has(key)),
    [],
    "known false-positive and zero-budget class rows must be disjoint",
  );
  for (const zeroClass of policy.zeroBudgetClasses) {
    assert.ok(zeroClass.ownerCommits.length > 0);
    for (const ownerCommit of zeroClass.ownerCommits) {
      assert.match(ownerCommit, /^[0-9a-f]{40}$/u);
    }
  }
}

function assertRealWorkspaceLintCensusCoverage(
  coveragePath: string,
  manifest: ExternalCorpusDifferentialManifestV1,
): void {
  const coverage = readJson<{
    readonly schemaVersion: string;
    readonly product: string;
    readonly entries: readonly { readonly id: string }[];
    readonly covered: readonly unknown[];
    readonly notCovered: readonly unknown[];
  }>(coveragePath);
  assert.equal(coverage.schemaVersion, "0");
  assert.equal(coverage.product, "omena-diff-test.real-workspace-lint-census-coverage");
  const localEntryIds = manifest.fixtures
    .filter(isLocalLintCensusEntry)
    .map(entryId)
    .sort((left, right) => left.localeCompare(right, "en"));
  assert.deepEqual(
    coverage.entries
      .map((entry) => entry.id)
      .sort((left, right) => left.localeCompare(right, "en")),
    localEntryIds,
    "lint census coverage must enumerate every local workspace",
  );
  assert.ok(coverage.covered.length > 0, "lint census coverage must name covered shapes");
  assert.ok(coverage.notCovered.length > 0, "lint census coverage must name deferred shapes");
  assertHonestCensusVocabulary(stableStringify(coverage));
}

function normalizeLintFinding(
  workspaceRoot: string,
  finding: ProductLintFindingV0,
): NormalizedLintFindingV0 {
  const relativePath = path.relative(workspaceRoot, finding.filePath).split(path.sep).join("/");
  assert.ok(isBoundedPath(relativePath), `finding escaped its workspace: ${finding.filePath}`);
  return {
    ruleId: finding.ruleId,
    filePath: relativePath,
    severity: finding.severity,
    range: finding.range,
    messageSha256: sha256(finding.message),
  };
}

function findingPinKey(finding: FindingPinV0): string {
  return stableStringify({
    filePath: finding.filePath,
    messageSha256: finding.messageSha256,
    range: finding.range,
    ruleId: finding.ruleId,
  });
}

function findingMatchesZeroBudgetClass(
  workspaceRoot: string,
  finding: NormalizedLintFindingV0,
  zeroClass: ZeroBudgetClassV0,
): boolean {
  if (finding.ruleId !== zeroClass.ruleId) return false;
  const sourceText = sourceTextForFinding(workspaceRoot, finding);
  if (zeroClass.matcher.kind === "specifier-prefixes") {
    const specifier = sourceText.replace(/["']/gu, "").trim();
    return zeroClass.matcher.prefixes.some((prefix) => specifier.startsWith(prefix));
  }
  const colon = sourceText.indexOf(":");
  if (colon < 0) return false;
  const declaration = {
    property: sourceText.slice(0, colon).trim().toLowerCase(),
    value: sourceText
      .slice(colon + 1)
      .replace(/;\s*$/u, "")
      .trim()
      .toLowerCase(),
  };
  return zeroClass.matcher.values.some(
    (value) => `${declaration.property}:${declaration.value}` === value.toLowerCase(),
  );
}

function sourceTextForFinding(workspaceRoot: string, finding: NormalizedLintFindingV0): string {
  const lines = readFileSync(path.join(workspaceRoot, finding.filePath), "utf8").split(/\r?\n/u);
  const { start, end } = finding.range;
  if (start.line === end.line) {
    return (lines[start.line] ?? "").slice(start.character, end.character);
  }
  return [
    (lines[start.line] ?? "").slice(start.character),
    ...lines.slice(start.line + 1, end.line),
    (lines[end.line] ?? "").slice(0, end.character),
  ].join("\n");
}

function workspaceContentDigest(workspaceRoot: string): string {
  const files = listWorkspaceCorpusFiles(workspaceRoot);
  const payload = files.map((filePath) => ({
    path: path.relative(workspaceRoot, filePath).split(path.sep).join("/"),
    sha256: sha256(readFileSync(filePath)),
  }));
  return sha256(stableStringify(payload));
}

function listWorkspaceCorpusFiles(root: string): string[] {
  const files: string[] = [];
  const visit = (directory: string): void => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if ([".cache", ".git", "node_modules", "target", "dist", "out"].includes(entry.name)) {
        continue;
      }
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(entryPath);
      } else if (/\.(?:css|scss|sass|less|jsx?|tsx?|json|toml)$/u.test(entry.name)) {
        files.push(entryPath);
      }
    }
  };
  visit(root);
  return files.sort((left, right) => left.localeCompare(right, "en"));
}

function resolveOmenaBinary(): string {
  const explicit = optionalValueAfter("--omena-bin") ?? process.env.OMENA_LINT_CENSUS_BINARY;
  if (explicit) {
    const resolved = path.resolve(repoRoot, explicit);
    assert.ok(existsSync(resolved), `omena lint census binary does not exist: ${resolved}`);
    return resolved;
  }
  run("cargo", [
    "build",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "--bin",
    "omena",
  ]);
  return path.join(
    repoRoot,
    "rust/target/debug",
    process.platform === "win32" ? "omena.exe" : "omena",
  );
}

function assertWorkspaceLintScanShape(): void {
  const lintSource = readFileSync(path.join(repoRoot, "rust/crates/omena-cli/src/lint.rs"), "utf8");
  const diagnosticsSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-cli/src/diagnostics.rs"),
    "utf8",
  );
  assert.equal(
    (lintSource.match(/workspace_source_diagnostics_summaries\(/gu) ?? []).length,
    1,
    "lint must batch source diagnostics once per workspace",
  );
  assert.equal(
    (lintSource.match(/\bsource_diagnostics_summary\(/gu) ?? []).length,
    0,
    "lint must not call the disk-loading source diagnostic path per file",
  );
  const batchBody = extractDelimitedBody(
    diagnosticsSource,
    "fn workspace_source_diagnostics_summaries",
  );
  const resolutionLoad = batchBody.indexOf("load_omena_query_workspace_style_resolution_inputs(");
  const sourceLoop = batchBody.indexOf("for source_path in source_paths");
  assert.equal(
    (batchBody.match(/load_omena_query_workspace_style_resolution_inputs\(/gu) ?? []).length,
    1,
    "workspace resolution inputs must load once before the source loop",
  );
  assert.ok(sourceLoop >= 0, "workspace source batch must retain its source loop");
  assert.ok(
    resolutionLoad >= 0 && resolutionLoad < sourceLoop,
    "workspace resolution inputs must be loaded before source iteration",
  );
}

function assertHonestCensusVocabulary(value: string): void {
  for (const forbidden of ["FP-free", "verified clean", "zero false positives"]) {
    assert.ok(!value.includes(forbidden), `census artifacts must not claim ${forbidden}`);
  }
}

function sumBy<T>(values: readonly T[], select: (value: T) => number): number {
  return values.reduce((sum, value) => sum + select(value), 0);
}

async function runFarm(entries: readonly ExternalCorpusEnvelopeV1[]): Promise<FactSetRecordV0[]> {
  const criteria = readManifest().selectionCriteria;
  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-oss-corpus-farm-"));
  try {
    const records: FactSetRecordV0[] = [];
    for (const entry of entries.filter(isPinnedRepositoryEntry)) {
      const id = entryId(entry);
      const checkoutDir = path.join(tempRoot, id);
      checkoutEntry(entry, checkoutDir);
      assertSelectedWorktreeWithinCeiling(entry, checkoutDir, criteria);
      records.push(
        await projectWorkspaceFactSet({
          id,
          repository: entry.source.repository,
          pin: entry.source.pin,
          checkoutDir,
        }),
      );
    }
    return records;
  } finally {
    rmSync(tempRoot, { force: true, recursive: true });
  }
}

function checkoutEntry(
  entry: ExternalCorpusEnvelopeV1 & { readonly source: PinnedRepositoryCorpusSourceV1 },
  checkoutDir: string,
): void {
  const sha = sourceSha(entry.source.pin);
  run("git", ["init", "-q", checkoutDir]);
  run("git", ["-C", checkoutDir, "remote", "add", "origin", entry.source.repository]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "init", "--no-cone"]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "set", ...entry.source.sparsePaths]);
  run("git", ["-C", checkoutDir, "fetch", "--depth", "1", "origin", sha]);
  run("git", ["-C", checkoutDir, "checkout", "-q", "--detach", "FETCH_HEAD"]);
  const actualSha = run("git", ["-C", checkoutDir, "rev-parse", "HEAD"]).stdout.trim();
  assert.equal(actualSha, sha, `${entryId(entry)} checkout did not resolve to the pinned sha`);
}

async function projectWorkspaceFactSet(input: {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly checkoutDir: string;
}): Promise<FactSetRecordV0> {
  let stdout = "";
  let stderr = "";
  let exitCode: number;
  try {
    exitCode = await runCheckerCli(
      [input.checkoutDir, "--preset", "ci", "--fail-on", "none", "--format", "json"],
      {
        stdout: (message) => {
          stdout += message;
        },
        stderr: (message) => {
          stderr += message;
        },
        cwd: () => repoRoot,
      },
    );
  } catch (error) {
    maybeWriteRawReproducer(input, {
      reason: `checker threw: ${(error as Error).message}`,
      exitCode: 1,
      stdoutJson: "not-checked",
    });
    throw error;
  }
  if (exitCode !== 0) {
    maybeWriteRawReproducer(input, {
      reason: `checker exited ${exitCode}\n${stderr}`,
      exitCode,
      stdoutJson: "not-checked",
    });
  }
  assert.equal(exitCode, 0, `${input.id} checker exited ${exitCode}\n${stderr}`);
  let report: CheckerReportV1;
  try {
    report = JSON.parse(stdout) as CheckerReportV1;
  } catch (error) {
    maybeWriteRawReproducer(input, {
      reason: `checker json parse failed: ${(error as Error).message}`,
      exitCode: 0,
      stdoutJson: "unparseable",
    });
    throw error;
  }
  const { facts, contentFactCount } = projectReportFacts(input.checkoutDir, report);
  assert.ok(contentFactCount > 0, `${input.id} produced an empty content-derived fact set`);
  const canonicalJson = stableStringify({
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.fact-set",
    id: input.id,
    repository: input.repository,
    pin: input.pin,
    facts,
  });
  return {
    id: input.id,
    repository: input.repository,
    pin: input.pin,
    factSetHash: sha256(canonicalJson),
    factCount: facts.length,
    canonicalJson,
  };
}

function projectReportFacts(
  workspaceRoot: string,
  report: CheckerReportV1,
): { readonly facts: readonly unknown[]; readonly contentFactCount: number } {
  const contentFacts: unknown[] = [];
  for (const filePath of report.sourceFiles ?? []) {
    contentFacts.push({
      kind: "source-file",
      path: relativeWorkspacePath(workspaceRoot, filePath),
    });
  }
  for (const filePath of report.styleFiles ?? []) {
    contentFacts.push({ kind: "style-file", path: relativeWorkspacePath(workspaceRoot, filePath) });
  }
  const facts: unknown[] = [...contentFacts];
  facts.push({
    kind: "summary",
    warnings: report.summary?.warnings ?? 0,
    hints: report.summary?.hints ?? 0,
    total: report.summary?.total ?? 0,
  });
  for (const finding of report.findings ?? []) {
    const findingFact = {
      kind: "finding",
      code: finding.code ?? "",
      severity: finding.severity ?? "",
      message: finding.message ?? "",
      filePath: finding.filePath ? relativeWorkspacePath(workspaceRoot, finding.filePath) : "",
      range: finding.range ?? null,
    };
    contentFacts.push(findingFact);
    facts.push(findingFact);
  }
  return {
    facts: facts.sort((left, right) =>
      stableStringify(left).localeCompare(stableStringify(right), "en"),
    ),
    contentFactCount: contentFacts.length,
  };
}

function buildReport(
  fresh: readonly BaselineRecordV0[],
  baselines: readonly BaselineRecordV0[],
): FarmReportV0 {
  const baselineById = new Map(baselines.map((baseline) => [baseline.id, baseline]));
  const reports = fresh.map((record): FarmEntryReportV0 => {
    const baseline = baselineById.get(record.id);
    const diffKind: DiffKind = !baseline
      ? "missing-baseline"
      : baseline.pin !== record.pin
        ? "pin-change"
        : baseline.factSetHash === record.factSetHash
          ? "pass"
          : "regression";
    return {
      id: record.id,
      repository: record.repository,
      pin: record.pin,
      factSetHash: record.factSetHash,
      factCount: record.factCount,
      diffKind,
    };
  });
  return {
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.report",
    entryCount: reports.length,
    comparedCount: reports.filter((entry) => entry.diffKind !== "missing-baseline").length,
    passCount: reports.filter((entry) => entry.diffKind === "pass").length,
    pinChangeCount: reports.filter((entry) => entry.diffKind === "pin-change").length,
    regressionCount: reports.filter((entry) => entry.diffKind === "regression").length,
    missingBaselineCount: reports.filter((entry) => entry.diffKind === "missing-baseline").length,
    reports,
  };
}

function classifierFixtureReport(): FarmReportV0 {
  const record = (id: string, pin: string, factSetHash: string): FactSetRecordV0 => ({
    id,
    repository: "https://github.com/example/project",
    pin,
    factSetHash,
    factCount: 1,
    canonicalJson: "[]",
  });
  const basePin = "example/project@0000000000000000000000000000000000000001";
  const nextPin = "example/project@0000000000000000000000000000000000000002";
  const baselines = [
    record("pass", basePin, "same"),
    record("pin-change", basePin, "same"),
    record("regression", basePin, "before"),
  ];
  const fresh = [
    record("pass", basePin, "same"),
    record("pin-change", nextPin, "same"),
    record("regression", basePin, "after"),
    record("missing-baseline", basePin, "new"),
  ];
  return buildReport(fresh, baselines);
}

function writeBaselines(records: readonly BaselineRecordV0[]): void {
  const ledger: BaselineLedgerV0 = {
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.baselines",
    generatedBy: "scripts/oss-corpus-farm.ts",
    baselines: records.map(({ id, repository, pin, factSetHash, factCount }) => ({
      id,
      repository,
      pin,
      factSetHash,
      factCount,
    })),
  };
  writeFileSync(baselinePath, `${JSON.stringify(ledger, null, 2)}\n`);
}

function writeReport(report: FarmReportV0): void {
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
}

function readManifest(): ExternalCorpusDifferentialManifestV1 {
  return readJson<ExternalCorpusDifferentialManifestV1>(manifestPath);
}

function assertManifest(manifest: ExternalCorpusDifferentialManifestV1): void {
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.oss-corpus-farm.manifest");
  assert.equal(manifest.mode, "pinned-repo-fact-set");
  assert.ok(manifest.fixtures.length > 0, "oss corpus farm manifest must not be empty");
  assertSelectionCriteria(manifest.selectionCriteria);
  assert.ok(isBoundedPath(manifest.lintCensus.policyPath));
  assert.ok(isBoundedPath(manifest.lintCensus.coveragePath));
  assert.ok(isBoundedPath(manifest.lintCensus.reportPath));
  assert.ok(manifest.lintCensus.fixedTimeBudgetSeconds > 0);
  assert.ok(manifest.lintCensus.perSourceFileBudgetSeconds > 0);
  const dialects = new Set(
    manifest.fixtures.filter(isPinnedRepositoryEntry).map((entry) => entry.dialect),
  );
  assert.ok(
    dialects.size >= manifest.selectionCriteria.minimumDialectCount,
    "oss corpus farm manifest must meet its dialect floor",
  );
  assert.ok(dialects.has("css"), "oss corpus farm manifest must include css");
  assert.ok(dialects.has("scss"), "oss corpus farm manifest must include scss");
  assert.ok(dialects.has("less"), "oss corpus farm manifest must include less");
  for (const entry of manifest.fixtures) {
    if (isPinnedRepositoryEntry(entry)) {
      assert.equal(entry.stage, "stage1-advisory");
      assert.equal(entry.expectationKind, "out-of-scope");
      assert.ok(entry.source.repository.startsWith("https://github.com/"));
      assert.ok(
        isSha(sourceSha(entry.source.pin)),
        `${entryId(entry)} must pin a 40-character sha`,
      );
      assert.ok(entry.source.sparsePaths.length > 0, `${entryId(entry)} must declare sparse paths`);
      assert.ok(
        entry.source.sparsePaths.length <= manifest.selectionCriteria.maxSparsePathCountPerEntry,
        `${entryId(entry)} sparse path count must stay within the manifest ceiling`,
      );
      assert.ok(
        entry.source.sparsePaths.every(isBoundedPath),
        `${entryId(entry)} sparse paths must stay bounded`,
      );
    } else {
      assert.equal(entry.stage, "stage2-blocking");
      assert.equal(entry.expectationKind, "finding-census");
      assert.ok(isBoundedPath(entry.source.workspacePath));
      assert.ok(
        existsSync(path.resolve(repoRoot, entry.source.workspacePath)),
        `${entryId(entry)} local workspace must exist`,
      );
    }
    const refs = [
      ...(entry.generation.oraclePinRefs ?? []),
      ...(entry.provenance?.oraclePinRefs ?? []),
    ];
    if (isPinnedRepositoryEntry(entry)) {
      assert.ok(refs.includes("spdx:MIT"), `${entryId(entry)} must record a permissive SPDX id`);
      assert.ok(
        refs.includes(`repo-sha:${sourceSha(entry.source.pin)}`),
        `${entryId(entry)} provenance sha must match source pin`,
      );
    } else {
      assert.ok(
        refs.includes(`repo-path:${entry.source.workspacePath}`),
        `${entryId(entry)} provenance path must match its local workspace`,
      );
    }
    assert.ok(entry.chunks.length > 0, `${entryId(entry)} must declare at least one chunk`);
    for (const chunk of entry.chunks) {
      assert.ok(isBoundedPath(chunk.path), `${chunk.chunkId} chunk path must stay bounded`);
      const chunkPath = path.join(farmRoot, chunk.path);
      assert.ok(existsSync(chunkPath), `${chunk.chunkId} chunk source must exist`);
      assert.ok(
        statSync(chunkPath).size <= manifest.selectionCriteria.maxChunkSourceBytes,
        `${chunk.chunkId} chunk source must stay within the manifest byte ceiling`,
      );
      assert.equal(sha256(readFileSync(chunkPath)), chunk.sha256);
      assert.ok(chunk.fixtureCount > 0, `${chunk.chunkId} fixture count must be non-zero`);
    }
    if (isLocalLintCensusEntry(entry)) {
      assertLocalWorkspaceSelection(entry);
    }
  }
}

function assertLocalWorkspaceSelection(
  entry: ExternalCorpusEnvelopeV1 & { readonly source: LocalWorkspaceCorpusSourceV1 },
): void {
  const selectionPath = path.resolve(repoRoot, entry.generation.selectionPath);
  assert.ok(existsSync(selectionPath), `${entryId(entry)} selection artifact must exist`);
  const committedSelection = readJson<LocalWorkspaceSelectionV0>(selectionPath);
  const selection = args.has("--inject-lint-census-selection-count-increase")
    ? { ...committedSelection, selectedFileCount: committedSelection.selectedFileCount + 1 }
    : committedSelection;
  assert.equal(selection.schemaVersion, "0");
  assert.equal(selection.product, "omena-diff-test.oss-corpus-farm.selection");
  assert.equal(selection.id, entryId(entry));
  assert.equal(selection.sourceKind, "local-workspace");
  assert.equal(selection.workspacePath, entry.source.workspacePath);
  const workspaceRoot = path.resolve(repoRoot, entry.source.workspacePath);
  const selectedFiles = listWorkspaceCorpusFiles(workspaceRoot);
  const selectedExtensions = new Set(selection.includedExtensions);
  assert.ok(selectedExtensions.size > 0, `${entryId(entry)} must name included extensions`);
  for (const filePath of selectedFiles) {
    const extension = path.extname(filePath).slice(1);
    assert.ok(
      selectedExtensions.has(extension),
      `${entryId(entry)} selection omitted the ${extension} extension`,
    );
  }
  assert.equal(
    selection.selectedFileCount,
    selectedFiles.length,
    `${entryId(entry)} selected file count must be source-derived`,
  );
  assert.equal(entry.chunks.length, 1, `${entryId(entry)} local selection must have one chunk`);
  assert.equal(
    entry.chunks[0]?.fixtureCount,
    selectedFiles.length,
    `${entryId(entry)} manifest fixture count must match its selected files`,
  );
  assert.equal(
    path.resolve(farmRoot, entry.chunks[0]?.path ?? ""),
    selectionPath,
    `${entryId(entry)} manifest and generation selection paths must agree`,
  );
}

function assertSelectionCriteria(criteria: OssCorpusFarmSelectionCriteriaV0): void {
  assert.ok(criteria.minimumDialectCount >= 3, "selection criteria must require all seed dialects");
  assert.ok(
    criteria.maxSparsePathCountPerEntry > 0,
    "selection criteria must bound sparse path count",
  );
  assert.ok(criteria.maxChunkSourceBytes > 0, "selection criteria must bound chunk bytes");
  assert.ok(
    criteria.maxSelectedWorktreeFiles > 0,
    "selection criteria must bound selected worktree files",
  );
  assert.ok(
    criteria.maxSelectedWorktreeBytes > 0,
    "selection criteria must bound selected worktree bytes",
  );
}

function assertSelectedWorktreeWithinCeiling(
  entry: ExternalCorpusEnvelopeV1,
  checkoutDir: string,
  criteria: OssCorpusFarmSelectionCriteriaV0,
): void {
  const files = listTrackedFiles(checkoutDir);
  const byteCount = files.reduce((sum, filePath) => sum + statSync(filePath).size, 0);
  assert.ok(
    files.length <= criteria.maxSelectedWorktreeFiles,
    `${entryId(entry)} selected worktree file count ${files.length} exceeds ${criteria.maxSelectedWorktreeFiles}`,
  );
  assert.ok(
    byteCount <= criteria.maxSelectedWorktreeBytes,
    `${entryId(entry)} selected worktree bytes ${byteCount} exceeds ${criteria.maxSelectedWorktreeBytes}`,
  );
}

function maybeWriteRawReproducer(
  input: {
    readonly id: string;
    readonly repository: string;
    readonly pin: string;
    readonly checkoutDir: string;
  },
  event: {
    readonly reason: string;
    readonly exitCode: number;
    readonly stdoutJson: "not-checked" | "unparseable";
  },
): void {
  if (process.env.OMENA_OSS_CORPUS_CAPTURE_RAW !== "1") return;
  const fixtureDir = path.join(rawCaptureRoot, input.id);
  mkdirSync(fixtureDir, { recursive: true });
  const files = listLoadedFiles(input.checkoutDir).slice(0, 64);
  const fixture = [
    `--- expect: raw-reproducer`,
    `repository: ${input.repository}`,
    `pin: ${input.pin}`,
    `minimization: raw`,
    `captureBacklog: PARKED-HRX-DDMIN`,
    `exitCode: ${event.exitCode}`,
    `stdoutJson: ${event.stdoutJson}`,
    `reason: ${event.reason.replace(/\r?\n/gu, " | ")}`,
    ...files.flatMap((filePath) => [
      `--- file: ${relativeWorkspacePath(input.checkoutDir, filePath)} encoding:hex`,
      readFileSync(filePath).toString("hex"),
    ]),
  ].join("\n");
  writeFileSync(path.join(fixtureDir, "fixture.omena"), `${fixture}\n`);
  updateRawCaptureManifest(input);
}

function updateRawCaptureManifest(input: {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
}): void {
  const manifestPathForCapture =
    rawCaptureRoot === regressionRoot
      ? regressionManifestPath
      : path.join(rawCaptureRoot, "manifest.json");
  const manifest: RegressionManifestV0 = existsSync(manifestPathForCapture)
    ? readJson<RegressionManifestV0>(manifestPathForCapture)
    : {
        schemaVersion: "0",
        product: "omena-diff-test.regression-corpus",
        fixtures: [],
      };
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.regression-corpus");
  const fixture: RegressionManifestFixtureV0 = {
    id: input.id,
    path: `${input.id}/fixture.omena`,
    status: "raw",
    sourceProvenance: {
      repository: input.repository,
      pin: input.pin,
    },
    minimization: "raw",
  };
  const fixtures = manifest.fixtures.filter((entry) => entry.id !== input.id);
  fixtures.push(fixture);
  mkdirSync(path.dirname(manifestPathForCapture), { recursive: true });
  writeFileSync(manifestPathForCapture, `${JSON.stringify({ ...manifest, fixtures }, null, 2)}\n`);
}

function listLoadedFiles(root: string): string[] {
  return listTrackedFiles(root).filter((filePath) =>
    /\.(?:css|scss|sass|less|jsx?|tsx?|json)$/u.test(filePath),
  );
}

function listTrackedFiles(root: string): string[] {
  const result = run("git", ["-C", root, "ls-files"]);
  return result.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((filePath) => path.join(root, filePath))
    .filter((filePath) => existsSync(filePath));
}

function entryId(entry: ExternalCorpusEnvelopeV1): string {
  const chunk = entry.chunks[0];
  assert.ok(chunk, "oss corpus farm entry must include a chunk id");
  return chunk.chunkId;
}

function isPinnedRepositoryEntry(
  entry: ExternalCorpusEnvelopeV1,
): entry is ExternalCorpusEnvelopeV1 & { readonly source: PinnedRepositoryCorpusSourceV1 } {
  return entry.source.kind === "pinned-repository";
}

function isLocalLintCensusEntry(
  entry: ExternalCorpusEnvelopeV1,
): entry is ExternalCorpusEnvelopeV1 & { readonly source: LocalWorkspaceCorpusSourceV1 } {
  return entry.source.kind === "local-workspace" && entry.expectationKind === "finding-census";
}

function sourceSha(pin: string): string {
  const sha = pin.split("@").at(-1) ?? "";
  assert.ok(isSha(sha), `invalid source pin sha: ${pin}`);
  return sha;
}

function isSha(value: string): boolean {
  return /^[0-9a-f]{40}$/u.test(value);
}

function isBoundedPath(value: string): boolean {
  return (
    value.length > 0 &&
    value !== "." &&
    !path.isAbsolute(value) &&
    !value.split(/[\\/]/u).includes("..")
  );
}

function relativeWorkspacePath(workspaceRoot: string, filePath: string): string {
  const relativePath = path.relative(workspaceRoot, filePath);
  return relativePath || ".";
}

function stableStringify(value: unknown): string {
  return JSON.stringify(sortForJson(value));
}

function extractDelimitedBody(source: string, marker: string): string {
  const declarationStart = source.indexOf(marker);
  assert.notEqual(declarationStart, -1, `missing ${marker}`);
  const bodyStart = source.indexOf("{", declarationStart);
  assert.notEqual(bodyStart, -1, `missing body for ${marker}`);
  let depth = 1;
  let cursor = bodyStart + 1;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated body for ${marker}`);
  return source.slice(bodyStart + 1, cursor - 1);
}

function sortForJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortForJson);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .sort(([left], [right]) => left.localeCompare(right, "en"))
      .map(([key, child]) => [key, sortForJson(child)]),
  );
}

function sha256(input: string | Buffer): string {
  return createHash("sha256").update(input).digest("hex");
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function resolveFarmPath(relativePath: string): string {
  assert.ok(isBoundedPath(relativePath), `farm artifact path must stay bounded: ${relativePath}`);
  return path.join(farmRoot, relativePath);
}

function valueAfter(flag: string): string {
  const index = process.argv.indexOf(flag);
  const value = process.argv[index + 1];
  assert.ok(value, `missing value for ${flag}`);
  return value;
}

function optionalValueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function run(command: string, args: readonly string[]): { readonly stdout: string } {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 64,
  });
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
