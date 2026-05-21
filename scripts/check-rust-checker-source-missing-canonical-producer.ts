import { deepStrictEqual } from "node:assert";
import path from "node:path";
import { buildCheckerBoundedGate } from "../packages/cme-checker/src";
import type { EngineInputV2 } from "../server/engine-core-ts/src/contracts";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  runShadowExpressionDomainFlowAnalysisInput,
  runShadowCheckerSourceMissingCanonicalCandidate,
  runShadowCheckerSourceMissingCanonicalProducer,
  type CheckerSourceMissingCanonicalProducerSignalV0,
} from "./rust-shadow-shared";

const REPO_ROOT = process.cwd();
const ESLINT_SMOKE_ROOT = path.join(REPO_ROOT, "test/_fixtures/eslint-plugin-smoke");

const SOURCE_MISSING_CORPUS: readonly ContractParityEntry[] = [
  {
    label: "eslint-smoke-missing-module",
    workspace: {
      workspaceRoot: ESLINT_SMOKE_ROOT,
      sourceFilePaths: [path.join(ESLINT_SMOKE_ROOT, "src/MissingModule.jsx")],
      styleFilePaths: [],
    },
    filters: {
      preset: "changed-source",
      category: "source",
      severity: "all",
      includeBundles: ["source-missing"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "eslint-smoke-missing-static-class",
    workspace: {
      workspaceRoot: ESLINT_SMOKE_ROOT,
      sourceFilePaths: [path.join(ESLINT_SMOKE_ROOT, "src/App.jsx")],
      styleFilePaths: [],
    },
    filters: {
      preset: "changed-source",
      category: "source",
      severity: "all",
      includeBundles: ["source-missing"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "eslint-smoke-missing-template-prefix",
    workspace: {
      workspaceRoot: ESLINT_SMOKE_ROOT,
      sourceFilePaths: [path.join(ESLINT_SMOKE_ROOT, "src/TemplatePrefix.jsx")],
      styleFilePaths: [],
    },
    filters: {
      preset: "changed-source",
      category: "source",
      severity: "all",
      includeBundles: ["source-missing"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "eslint-smoke-missing-resolved-class-values",
    workspace: {
      workspaceRoot: ESLINT_SMOKE_ROOT,
      sourceFilePaths: [path.join(ESLINT_SMOKE_ROOT, "src/Dynamic.jsx")],
      styleFilePaths: [],
    },
    filters: {
      preset: "changed-source",
      category: "source",
      severity: "all",
      includeBundles: ["source-missing"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "eslint-smoke-missing-resolved-class-domain",
    workspace: {
      workspaceRoot: ESLINT_SMOKE_ROOT,
      sourceFilePaths: [path.join(ESLINT_SMOKE_ROOT, "src/DynamicDomain.jsx")],
      styleFilePaths: [],
    },
    filters: {
      preset: "changed-source",
      category: "source",
      severity: "all",
      includeBundles: ["source-missing"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
] as const;

void (async () => {
  for (const entry of SOURCE_MISSING_CORPUS) {
    process.stdout.write(`== rust-checker-source-missing-producer:${entry.label} ==\n`);
    // oxlint-disable-next-line no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    // oxlint-disable-next-line no-await-in-loop
    const flowSummary = await runShadowExpressionDomainFlowAnalysisInput(
      (snapshot as { readonly input: EngineInputV2 }).input,
    );
    // oxlint-disable-next-line no-await-in-loop
    const canonicalCandidate = await runShadowCheckerSourceMissingCanonicalCandidate(snapshot);
    // oxlint-disable-next-line no-await-in-loop
    const actual = await runShadowCheckerSourceMissingCanonicalProducer(snapshot);

    const expected: CheckerSourceMissingCanonicalProducerSignalV0 = {
      schemaVersion: "0",
      inputVersion: canonicalCandidate.inputVersion,
      canonicalCandidate,
      flowEvidence: {
        schemaVersion: "0",
        product: "engine-input-producers.expression-domain-flow-analysis",
        inputVersion: flowSummary.inputVersion,
        graphCount: flowSummary.analyses.length,
        nodeCount: flowSummary.analyses.reduce(
          (sum, flowEntry) => sum + flowEntry.analysis.nodes.length,
          0,
        ),
        convergedGraphCount: flowSummary.analyses.filter(
          (flowEntry) => flowEntry.analysis.converged,
        ).length,
        unconvergedGraphCount: flowSummary.analyses.filter(
          (flowEntry) => !flowEntry.analysis.converged,
        ).length,
        maxIterationCount: Math.max(
          0,
          ...flowSummary.analyses.map((flowEntry) => flowEntry.analysis.iterationCount),
        ),
      },
      boundedCheckerGate: buildCheckerBoundedGate("source-missing"),
    };

    deepStrictEqual(
      actual,
      expected,
      `${entry.label}: checker source-missing canonical producer mismatch`,
    );
    process.stdout.write(
      `findings=${actual.canonicalCandidate.summary.total} flowGraphs=${actual.flowEvidence.graphCount} releaseGate=${actual.boundedCheckerGate.includedInRustReleaseBundle}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
