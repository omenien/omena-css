import { deepStrictEqual, strict as assert } from "node:assert";
import path from "node:path";
import {
  buildCheckerBoundedGate,
  deriveCheckerCanonicalCandidate,
} from "../packages/cme-checker/src";
import { runCheckerCli } from "../server/checker-cli/src";
import type { EngineInputV2 } from "../server/engine-core-ts/src/contracts";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  runShadowExpressionDomainFlowAnalysisInput,
  runShadowCheckerSourceMissingCanonicalCandidate,
  runShadowCheckerSourceMissingCanonicalProducer,
  runShadowCheckerStyleRecoveryCanonicalCandidate,
  runShadowCheckerStyleRecoveryCanonicalProducer,
  runShadowCheckerStyleUnusedCanonicalCandidate,
  runShadowCheckerStyleUnusedCanonicalProducer,
  type CheckerSourceMissingCanonicalCandidateBundleV0,
  type CheckerSourceMissingCanonicalProducerSignalV0,
  type CheckerStyleRecoveryCanonicalCandidateBundleV0,
  type CheckerStyleRecoveryCanonicalProducerSignalV0,
  type CheckerStyleUnusedCanonicalProducerSignalV0,
  type ExpressionDomainFlowAnalysisV0,
} from "./rust-shadow-shared";
import { deriveTsCheckerStyleUnusedCanonicalCandidate } from "./rust-checker-style-unused-shared";

const STYLE_RECOVERY_CODES = new Set([
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "missing-keyframes",
  "missing-sass-symbol",
]);

const SOURCE_MISSING_CODES = new Set([
  "missing-module",
  "missing-static-class",
  "missing-template-prefix",
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
]);

const REPO_ROOT = process.cwd();
const WORKSPACE_ROOT = path.join(REPO_ROOT, "test/_fixtures/real-project-checker-corpus");

const STYLE_RECOVERY_ENTRY: ContractParityEntry = {
  label: "real-project-dashboard-card",
  workspace: {
    workspaceRoot: WORKSPACE_ROOT,
    sourceFilePaths: [path.join(WORKSPACE_ROOT, "DashboardCard.tsx")],
    styleFilePaths: [
      path.join(WORKSPACE_ROOT, "DashboardCard.module.scss"),
      path.join(WORKSPACE_ROOT, "DashboardCardBase.module.scss"),
    ],
  },
  filters: {
    preset: "ci",
    category: "style",
    severity: "all",
    includeBundles: ["style-recovery"],
    includeCodes: [],
    excludeCodes: [],
  },
};

const SOURCE_MISSING_ENTRY: ContractParityEntry = {
  label: "real-project-nav-pill",
  workspace: {
    workspaceRoot: WORKSPACE_ROOT,
    sourceFilePaths: [path.join(WORKSPACE_ROOT, "NavPill.tsx")],
    styleFilePaths: [path.join(WORKSPACE_ROOT, "NavPill.module.scss")],
  },
  filters: {
    preset: "ci",
    category: "source",
    severity: "all",
    includeBundles: ["source-missing"],
    includeCodes: [],
    excludeCodes: [],
  },
};

const STYLE_UNUSED_ENTRY: ContractParityEntry = {
  label: "real-project-dashboard-card-unused",
  workspace: {
    workspaceRoot: WORKSPACE_ROOT,
    sourceFilePaths: [path.join(WORKSPACE_ROOT, "DashboardCard.tsx")],
    styleFilePaths: [path.join(WORKSPACE_ROOT, "DashboardCard.module.scss")],
  },
  filters: {
    preset: "ci",
    category: "style",
    severity: "all",
    includeBundles: ["style-unused"],
    includeCodes: [],
    excludeCodes: [],
  },
};

void (async () => {
  const styleSnapshot = await buildContractParitySnapshot(STYLE_RECOVERY_ENTRY);
  const sourceSnapshot = await buildContractParitySnapshot(SOURCE_MISSING_ENTRY);
  const unusedSnapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);

  const expectedStyleCandidate = deriveTsCheckerStyleRecoveryCanonicalCandidate(styleSnapshot);
  const actualStyleCandidate = await runShadowCheckerStyleRecoveryCanonicalCandidate(styleSnapshot);
  deepStrictEqual(
    actualStyleCandidate,
    expectedStyleCandidate,
    "real-project-dashboard-card: checker style-recovery canonical candidate mismatch",
  );
  assert.equal(actualStyleCandidate.summary.total, 1);
  assert.equal(actualStyleCandidate.findings[0]?.code, "missing-composed-selector");

  const expectedSourceCandidate = deriveTsCheckerSourceMissingCanonicalCandidate(sourceSnapshot);
  const actualSourceCandidate =
    await runShadowCheckerSourceMissingCanonicalCandidate(sourceSnapshot);
  deepStrictEqual(
    actualSourceCandidate,
    expectedSourceCandidate,
    "real-project-nav-pill: checker source-missing canonical candidate mismatch",
  );
  assert.equal(actualSourceCandidate.summary.total, 1);
  assert.equal(actualSourceCandidate.findings[0]?.code, "missing-static-class");

  const expectedUnusedCandidate = deriveTsCheckerStyleUnusedCanonicalCandidate(unusedSnapshot);
  const actualUnusedCandidate = await runShadowCheckerStyleUnusedCanonicalCandidate(unusedSnapshot);
  deepStrictEqual(
    actualUnusedCandidate,
    expectedUnusedCandidate,
    "real-project-dashboard-card-unused: checker style-unused canonical candidate mismatch",
  );
  assert.equal(actualUnusedCandidate.summary.total, 1);
  assert.equal(actualUnusedCandidate.findings[0]?.code, "unused-selector");

  const actualStyleProducer = await runShadowCheckerStyleRecoveryCanonicalProducer(styleSnapshot);
  deepStrictEqual(actualStyleProducer, {
    schemaVersion: "0",
    inputVersion: expectedStyleCandidate.inputVersion,
    canonicalCandidate: expectedStyleCandidate,
    boundedCheckerGate: buildCheckerBoundedGate("style-recovery"),
  } satisfies CheckerStyleRecoveryCanonicalProducerSignalV0);

  const sourceFlowSummary = await runShadowExpressionDomainFlowAnalysisInput(
    (sourceSnapshot as { readonly input: EngineInputV2 }).input,
  );
  const actualSourceProducer = await runShadowCheckerSourceMissingCanonicalProducer(sourceSnapshot);
  deepStrictEqual(actualSourceProducer, {
    schemaVersion: "0",
    inputVersion: expectedSourceCandidate.inputVersion,
    canonicalCandidate: expectedSourceCandidate,
    flowEvidence: deriveFlowEvidence(sourceFlowSummary),
    boundedCheckerGate: buildCheckerBoundedGate("source-missing"),
  } satisfies CheckerSourceMissingCanonicalProducerSignalV0);

  const actualUnusedProducer = await runShadowCheckerStyleUnusedCanonicalProducer(unusedSnapshot);
  deepStrictEqual(actualUnusedProducer, {
    schemaVersion: "0",
    inputVersion: expectedUnusedCandidate.inputVersion,
    canonicalCandidate: expectedUnusedCandidate,
    boundedCheckerGate: buildCheckerBoundedGate("style-unused"),
  } satisfies CheckerStyleUnusedCanonicalProducerSignalV0);

  const styleConsumerPayload = await runRustConsumerCheck({
    cwd: WORKSPACE_ROOT,
    sourceFiles: ["DashboardCard.tsx"],
    styleFiles: ["DashboardCard.module.scss", "DashboardCardBase.module.scss"],
    includeBundle: "style-recovery",
    flag: "--rust-style-recovery-consumer",
  });
  assert.equal(styleConsumerPayload.summary.total, 1);
  assert.equal(styleConsumerPayload.findings[0]?.code, "missing-composed-selector");
  assert.equal(styleConsumerPayload.rustStyleRecoveryConsistency.findingsMatch, true);
  assert.equal(styleConsumerPayload.rustStyleRecoveryConsistency.countsMatch, true);

  const sourceConsumerPayload = await runRustConsumerCheck({
    cwd: WORKSPACE_ROOT,
    sourceFiles: ["NavPill.tsx"],
    styleFiles: ["NavPill.module.scss"],
    includeBundle: "source-missing",
    flag: "--rust-source-missing-consumer",
  });
  assert.equal(sourceConsumerPayload.summary.total, 1);
  assert.equal(sourceConsumerPayload.findings[0]?.code, "missing-static-class");
  assert.equal(sourceConsumerPayload.rustSourceMissingConsistency.findingsMatch, true);
  assert.equal(sourceConsumerPayload.rustSourceMissingConsistency.countsMatch, true);

  const unusedConsumerPayload = await runRustConsumerCheck({
    cwd: WORKSPACE_ROOT,
    sourceFiles: ["DashboardCard.tsx"],
    styleFiles: ["DashboardCard.module.scss"],
    includeBundle: "style-unused",
    flag: "--rust-style-unused-consumer",
  });
  assert.equal(unusedConsumerPayload.summary.total, 1);
  assert.equal(unusedConsumerPayload.findings[0]?.code, "unused-selector");
  assert.equal(unusedConsumerPayload.rustStyleUnusedConsistency.findingsMatch, true);
  assert.equal(unusedConsumerPayload.rustStyleUnusedConsistency.countsMatch, true);

  process.stdout.write(
    [
      "== rust-checker-real-project-bounded:style-recovery ==",
      `label=${STYLE_RECOVERY_ENTRY.label}`,
      `findings=${actualStyleCandidate.summary.total}`,
      `code=${actualStyleCandidate.findings[0]?.code}`,
      "consistent=true",
      "",
    ].join("\n"),
  );
  process.stdout.write(
    [
      "== rust-checker-real-project-bounded:source-missing ==",
      `label=${SOURCE_MISSING_ENTRY.label}`,
      `findings=${actualSourceCandidate.summary.total}`,
      `code=${actualSourceCandidate.findings[0]?.code}`,
      "consistent=true",
      "",
    ].join("\n"),
  );
  process.stdout.write(
    [
      "== rust-checker-real-project-bounded:style-unused ==",
      `label=${STYLE_UNUSED_ENTRY.label}`,
      `findings=${actualUnusedCandidate.summary.total}`,
      `code=${actualUnusedCandidate.findings[0]?.code}`,
      "consistent=true",
      "",
    ].join("\n"),
  );
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

function deriveTsCheckerStyleRecoveryCanonicalCandidate(
  snapshot: Awaited<ReturnType<typeof buildContractParitySnapshot>>,
): CheckerStyleRecoveryCanonicalCandidateBundleV0 {
  return deriveCheckerCanonicalCandidate(snapshot, {
    bundle: "style-recovery",
    category: "style",
    codes: STYLE_RECOVERY_CODES,
    extraFields: ["analysisReason", "valueCertaintyShapeLabel"],
  }) as CheckerStyleRecoveryCanonicalCandidateBundleV0;
}

function deriveTsCheckerSourceMissingCanonicalCandidate(
  snapshot: Awaited<ReturnType<typeof buildContractParitySnapshot>>,
): CheckerSourceMissingCanonicalCandidateBundleV0 {
  return deriveCheckerCanonicalCandidate(snapshot, {
    bundle: "source-missing",
    category: "source",
    codes: SOURCE_MISSING_CODES,
    extraFields: ["analysisReason", "valueCertaintyShapeLabel", "valueDomainDerivation"],
  }) as CheckerSourceMissingCanonicalCandidateBundleV0;
}

function deriveFlowEvidence(flowSummary: ExpressionDomainFlowAnalysisV0) {
  const convergedGraphCount = flowSummary.analyses.filter(
    (flowEntry) => flowEntry.analysis.converged,
  ).length;
  return {
    schemaVersion: "0",
    product: "engine-input-producers.expression-domain-flow-analysis",
    inputVersion: flowSummary.inputVersion,
    graphCount: flowSummary.analyses.length,
    nodeCount: flowSummary.analyses.reduce(
      (sum, flowEntry) => sum + flowEntry.analysis.nodes.length,
      0,
    ),
    convergedGraphCount,
    unconvergedGraphCount: flowSummary.analyses.length - convergedGraphCount,
    maxIterationCount: Math.max(
      0,
      ...flowSummary.analyses.map((flowEntry) => flowEntry.analysis.iterationCount),
    ),
  };
}

async function runRustConsumerCheck(input: {
  readonly cwd: string;
  readonly sourceFiles: readonly string[];
  readonly styleFiles: readonly string[];
  readonly includeBundle: "style-recovery" | "source-missing" | "style-unused";
  readonly flag:
    | "--rust-style-recovery-consumer"
    | "--rust-source-missing-consumer"
    | "--rust-style-unused-consumer";
}): Promise<any> {
  const stdout: string[] = [];
  const stderr: string[] = [];
  const args = [
    input.cwd,
    "--preset",
    "ci",
    "--severity",
    "all",
    ...input.sourceFiles.flatMap((file) => ["--source-file", file]),
    ...input.styleFiles.flatMap((file) => ["--style-file", file]),
    "--include-bundle",
    input.includeBundle,
    "--format",
    "json",
    "--fail-on",
    "none",
    input.flag,
  ];

  const exitCode = await runCheckerCli(args, {
    stdout: (message) => stdout.push(message),
    stderr: (message) => stderr.push(message),
    cwd: () => input.cwd,
  });

  assert.equal(exitCode, 0, `${input.includeBundle}: expected zero exit`);
  assert.equal(stderr.join(""), "", `${input.includeBundle}: unexpected stderr`);
  return JSON.parse(stdout.join(""));
}
