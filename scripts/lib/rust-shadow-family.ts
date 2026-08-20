// g131-S6: the rust-shadow thin-driver family, collapsed into one table.
// 42 former single-file drivers (14 shared-corpus COLLAPSE + 28 own-corpus
// RELOCATE) keyed by their former script slug; the driver script executes
// exactly one member per invocation so gate ids and outputs are unchanged.

import {
  assertCheckerCanonicalCandidateEqual,
  buildCheckerBoundedGate,
  deriveCheckerCanonicalCandidate,
} from "../../packages/cme-checker/src";
import { runCheckerCli } from "../../server/checker-cli/src";
import type { EngineInputV2, StringTypeFactsV2 } from "../../server/engine-core-ts/src/contracts";
import type { ContractParityEntry } from "../contract-parity-corpus-v1";
import {
  CONTRACT_PARITY_CORPUS_V2,
  type ContractParityAuthoredMembershipV2,
} from "../contract-parity-corpus-v2";
import { buildContractParitySnapshot } from "../contract-parity-runtime";
import {
  STYLE_UNUSED_ENTRY,
  STYLE_UNUSED_WORKSPACE_ROOT,
  deriveTsCheckerStyleUnusedCanonicalCandidate,
} from "../rust-checker-style-unused-shared";
import {
  assertExpressionDomainCandidatesMatch,
  assertExpressionDomainCanonicalCandidateBundleMatch,
  assertExpressionDomainCanonicalProducerSignalMatch,
  assertExpressionDomainFragmentsMatch,
  assertExpressionDomainPlanSummaryMatch,
  assertExpressionSemanticsCandidatesMatch,
  assertExpressionSemanticsCanonicalCandidateBundleMatch,
  assertExpressionSemanticsCanonicalProducerSignalMatch,
  assertExpressionSemanticsEvaluatorCandidatesMatch,
  assertExpressionSemanticsFragmentsMatch,
  assertExpressionSemanticsMatchFragmentsMatch,
  assertExpressionSemanticsQueryFragmentsMatch,
  assertQueryPlanSummaryMatch,
  assertSelectorUsageFragmentsMatch,
  assertSelectorUsagePlanSummaryMatch,
  assertSelectorUsageQueryFragmentsMatch,
  assertSemanticCanonicalCandidateBundleMatch,
  assertSemanticCanonicalProducerSignalMatch,
  assertSemanticEvaluatorCandidatesMatch,
  assertShadowSummaryMatch,
  assertSourceResolutionCandidatesMatch,
  assertSourceResolutionCanonicalCandidateBundleMatch,
  assertSourceResolutionCanonicalProducerSignalMatch,
  assertSourceResolutionEvaluatorCandidatesMatch,
  assertSourceResolutionFragmentsMatch,
  assertSourceResolutionMatchFragmentsMatch,
  assertSourceResolutionPlanSummaryMatch,
  assertSourceResolutionQueryFragmentsMatch,
  assertSourceSideCanonicalCandidateBundleMatch,
  assertSourceSideCanonicalProducerSignalMatch,
  assertSourceSideEvaluatorCandidatesMatch,
  deriveTsExpressionDomainCandidates,
  deriveTsExpressionDomainCanonicalCandidateBundle,
  deriveTsExpressionDomainCanonicalProducerSignal,
  deriveTsExpressionDomainEvaluatorCandidates,
  deriveTsExpressionDomainFragments,
  deriveTsExpressionDomainPlanSummary,
  deriveTsExpressionSemanticsCandidates,
  deriveTsExpressionSemanticsCanonicalCandidateBundle,
  deriveTsExpressionSemanticsCanonicalProducerSignal,
  deriveTsExpressionSemanticsEvaluatorCandidates,
  deriveTsExpressionSemanticsFragments,
  deriveTsExpressionSemanticsMatchFragments,
  deriveTsExpressionSemanticsQueryFragments,
  deriveTsQueryPlanSummary,
  deriveTsSelectorUsageFragments,
  deriveTsSelectorUsagePlanSummary,
  deriveTsSelectorUsageQueryFragments,
  deriveTsSemanticCanonicalCandidateBundle,
  deriveTsSemanticCanonicalProducerSignal,
  deriveTsSemanticEvaluatorCandidates,
  deriveTsShadowSummary,
  deriveTsSourceResolutionCandidates,
  deriveTsSourceResolutionCanonicalCandidateBundle,
  deriveTsSourceResolutionCanonicalProducerSignal,
  deriveTsSourceResolutionEvaluatorCandidates,
  deriveTsSourceResolutionFragments,
  deriveTsSourceResolutionMatchFragments,
  deriveTsSourceResolutionPlanSummary,
  deriveTsSourceResolutionQueryFragments,
  deriveTsSourceSideCanonicalCandidateBundle,
  deriveTsSourceSideCanonicalProducerSignal,
  deriveTsSourceSideEvaluatorCandidates,
  deriveTsTypeFactInputSummary,
  runShadow,
  runShadowCheckerSourceMissingCanonicalCandidate,
  runShadowCheckerSourceMissingCanonicalProducer,
  runShadowCheckerStyleRecoveryCanonicalCandidate,
  runShadowCheckerStyleRecoveryCanonicalProducer,
  runShadowCheckerStyleUnusedCanonicalCandidate,
  runShadowCheckerStyleUnusedCanonicalProducer,
  runShadowExpressionDomainCandidatesInput,
  runShadowExpressionDomainCanonicalCandidateInput,
  runShadowExpressionDomainCanonicalProducerInput,
  runShadowExpressionDomainEvaluatorCandidatesInput,
  runShadowExpressionDomainFlowAnalysisInput,
  runShadowExpressionDomainFragmentsInput,
  runShadowExpressionDomainInput,
  runShadowExpressionSemanticsCandidatesInput,
  runShadowExpressionSemanticsCanonicalCandidateInput,
  runShadowExpressionSemanticsCanonicalProducerInput,
  runShadowExpressionSemanticsEvaluatorCandidatesInput,
  runShadowExpressionSemanticsFragmentsInput,
  runShadowExpressionSemanticsMatchFragmentsInput,
  runShadowExpressionSemanticsQueryFragmentsInput,
  runShadowQueryPlanInput,
  runShadowSelectorUsageFragmentsInput,
  runShadowSelectorUsagePlanInput,
  runShadowSelectorUsageQueryFragmentsInput,
  runShadowSemanticCanonicalCandidateInput,
  runShadowSemanticCanonicalProducerInput,
  runShadowSemanticEvaluatorCandidatesInput,
  runShadowSourceResolutionCandidatesInput,
  runShadowSourceResolutionCanonicalCandidateInput,
  runShadowSourceResolutionCanonicalProducerInput,
  runShadowSourceResolutionEvaluatorCandidatesInput,
  runShadowSourceResolutionFragmentsInput,
  runShadowSourceResolutionMatchFragmentsInput,
  runShadowSourceResolutionPlanInput,
  runShadowSourceResolutionQueryFragmentsInput,
  runShadowSourceSideCanonicalCandidateInput,
  runShadowSourceSideCanonicalProducerInput,
  runShadowSourceSideEvaluatorCandidatesInput,
  runShadowTypeFactInput,
  type CheckerSourceMissingCanonicalCandidateBundleV0,
  type CheckerSourceMissingCanonicalProducerSignalV0,
  type CheckerStyleRecoveryCanonicalCandidateBundleV0,
  type CheckerStyleRecoveryCanonicalProducerSignalV0,
  type CheckerStyleUnusedCanonicalProducerSignalV0,
  type TypeFactInputSummaryV0,
} from "../rust-shadow-shared";
import { deepStrictEqual, strict as assert } from "node:assert";
import path from "node:path";

async function run_rust_checker_source_missing_canonical_candidate(): Promise<void> {
  const SOURCE_MISSING_CODES = new Set([
    "missing-module",
    "missing-static-class",
    "missing-template-prefix",
    "missing-resolved-class-values",
    "missing-resolved-class-domain",
  ]);

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

  await (async () => {
    for (const entry of SOURCE_MISSING_CORPUS) {
      process.stdout.write(`== rust-checker-source-missing:${entry.label} ==\n`);
      // oxlint-disable-next-line no-await-in-loop
      const snapshot = await buildContractParitySnapshot(entry);
      const expected = deriveTsCheckerSourceMissingCanonicalCandidate(snapshot);
      // oxlint-disable-next-line no-await-in-loop
      const actual = await runShadowCheckerSourceMissingCanonicalCandidate(snapshot);
      assertCheckerCanonicalCandidateEqual(
        actual,
        expected,
        `${entry.label}: checker source-missing canonical candidate mismatch`,
      );
      process.stdout.write(
        `findings=${actual.summary.total} files=${actual.distinctFileCount} codes=${JSON.stringify(actual.codeCounts)}\n\n`,
      );
    }
  })().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });

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
}

async function run_rust_checker_source_missing_canonical_producer(): Promise<void> {
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

  await (async () => {
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
}

async function run_rust_checker_style_recovery_canonical_candidate(): Promise<void> {
  const STYLE_RECOVERY_CODES = new Set([
    "missing-composed-module",
    "missing-composed-selector",
    "missing-value-module",
    "missing-imported-value",
    "missing-keyframes",
    "missing-sass-symbol",
  ]);

  const REPO_ROOT = process.cwd();
  const STYLELINT_SMOKE_ROOT = path.join(REPO_ROOT, "test/_fixtures/stylelint-plugin-smoke");

  const STYLE_RECOVERY_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "stylelint-smoke-composes-missing-module",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingModule.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-composes-missing-selector",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingSelector.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-value-missing-module",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingModule.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-value-missing-imported",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingImported.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-keyframes-missing",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/KeyframesMissing.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
  ] as const;

  for (const entry of STYLE_RECOVERY_CORPUS) {
    process.stdout.write(`== rust-checker-style-recovery:${entry.label} ==\n`);
    // oxlint-disable-next-line no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsCheckerStyleRecoveryCanonicalCandidate(snapshot);
    // oxlint-disable-next-line no-await-in-loop
    const actual = await runShadowCheckerStyleRecoveryCanonicalCandidate(snapshot);
    assertCheckerCanonicalCandidateEqual(
      actual,
      expected,
      `${entry.label}: checker style-recovery canonical candidate mismatch`,
    );
    process.stdout.write(
      `findings=${actual.summary.total} files=${actual.distinctFileCount} codes=${JSON.stringify(actual.codeCounts)}\n\n`,
    );
  }

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
}

async function run_rust_checker_style_recovery_canonical_producer(): Promise<void> {
  const REPO_ROOT = process.cwd();
  const STYLELINT_SMOKE_ROOT = path.join(REPO_ROOT, "test/_fixtures/stylelint-plugin-smoke");

  const STYLE_RECOVERY_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "stylelint-smoke-composes-missing-module",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingModule.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-composes-missing-selector",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingSelector.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-value-missing-module",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingModule.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-value-missing-imported",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingImported.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    {
      label: "stylelint-smoke-keyframes-missing",
      workspace: {
        workspaceRoot: STYLELINT_SMOKE_ROOT,
        sourceFilePaths: [],
        styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/KeyframesMissing.module.css")],
      },
      filters: {
        preset: "changed-style",
        category: "style",
        severity: "all",
        includeBundles: ["style-recovery"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
  ] as const;

  await (async () => {
    for (const entry of STYLE_RECOVERY_CORPUS) {
      process.stdout.write(`== rust-checker-style-recovery-producer:${entry.label} ==\n`);
      // oxlint-disable-next-line no-await-in-loop
      const snapshot = await buildContractParitySnapshot(entry);
      // oxlint-disable-next-line no-await-in-loop
      const canonicalCandidate = await runShadowCheckerStyleRecoveryCanonicalCandidate(snapshot);
      // oxlint-disable-next-line no-await-in-loop
      const actual = await runShadowCheckerStyleRecoveryCanonicalProducer(snapshot);

      const expected: CheckerStyleRecoveryCanonicalProducerSignalV0 = {
        schemaVersion: "0",
        inputVersion: canonicalCandidate.inputVersion,
        canonicalCandidate,
        boundedCheckerGate: buildCheckerBoundedGate("style-recovery"),
      };

      deepStrictEqual(
        actual,
        expected,
        `${entry.label}: checker style-recovery canonical producer mismatch`,
      );
      process.stdout.write(
        `findings=${actual.canonicalCandidate.summary.total} releaseGate=${actual.boundedCheckerGate.includedInRustReleaseBundle}\n\n`,
      );
    }
  })().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}

async function run_rust_checker_style_unused_canonical_candidate(): Promise<void> {
  await (async () => {
    process.stdout.write(`== rust-checker-style-unused:${STYLE_UNUSED_ENTRY.label} ==\n`);
    const snapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);
    const expected = deriveTsCheckerStyleUnusedCanonicalCandidate(snapshot);
    const actual = await runShadowCheckerStyleUnusedCanonicalCandidate(snapshot);
    assertCheckerCanonicalCandidateEqual(
      actual,
      expected,
      `${STYLE_UNUSED_ENTRY.label}: checker style-unused canonical candidate mismatch`,
    );
    process.stdout.write(
      `findings=${actual.summary.total} files=${actual.distinctFileCount} codes=${JSON.stringify(actual.codeCounts)}\n\n`,
    );
  })().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}

async function run_rust_checker_style_unused_canonical_producer(): Promise<void> {
  await (async () => {
    process.stdout.write(`== rust-checker-style-unused-producer:${STYLE_UNUSED_ENTRY.label} ==\n`);
    const snapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);
    const canonicalCandidate = deriveTsCheckerStyleUnusedCanonicalCandidate(snapshot);
    const actual = await runShadowCheckerStyleUnusedCanonicalProducer(snapshot);

    const expected: CheckerStyleUnusedCanonicalProducerSignalV0 = {
      schemaVersion: "0",
      inputVersion: canonicalCandidate.inputVersion,
      canonicalCandidate,
      boundedCheckerGate: buildCheckerBoundedGate("style-unused"),
    };

    deepStrictEqual(actual, expected, "checker style-unused canonical producer mismatch");
    process.stdout.write(
      `findings=${actual.canonicalCandidate.summary.total} releaseGate=${actual.boundedCheckerGate.includedInRustReleaseBundle}\n\n`,
    );
  })().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}

async function run_rust_checker_style_unused_consumer_boundary(): Promise<void> {
  await (async () => {
    const stdout: string[] = [];
    const stderr: string[] = [];

    const exitCode = await runCheckerCli(
      [
        STYLE_UNUSED_WORKSPACE_ROOT,
        "--source-file",
        "src/App.tsx",
        "--style-file",
        "src/App.module.css",
        "--preset",
        "changed-style",
        "--include-bundle",
        "style-unused",
        "--format",
        "json",
        "--fail-on",
        "none",
        "--rust-style-unused-consumer",
      ],
      {
        stdout: (message) => stdout.push(message),
        stderr: (message) => stderr.push(message),
        cwd: () => STYLE_UNUSED_WORKSPACE_ROOT,
      },
    );

    assert.equal(exitCode, 0, "expected zero exit");
    assert.equal(stderr.join(""), "", "unexpected stderr");
    const payload = JSON.parse(stdout.join(""));
    assert.equal(payload.summary.total, 1, "expected one style-unused finding");
    assert.equal(payload.findings[0]?.code, "unused-selector", "unexpected finding code");
    assert.ok(payload.rustStyleUnusedCanonicalProducer, "missing rustStyleUnusedCanonicalProducer");
    assert.ok(payload.rustStyleUnusedConsistency, "missing rustStyleUnusedConsistency");

    const snapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);
    const expectedCandidate = deriveTsCheckerStyleUnusedCanonicalCandidate(snapshot);
    const actualProducer = await runShadowCheckerStyleUnusedCanonicalProducer(snapshot);
    assert.deepEqual(actualProducer.canonicalCandidate, expectedCandidate);
    assert.equal(actualProducer.canonicalCandidate.summary.total, payload.summary.total);
    assert.equal(actualProducer.canonicalCandidate.findings[0]?.code, payload.findings[0]?.code);
    assert.deepEqual(
      payload.rustStyleUnusedCanonicalProducer.canonicalCandidate,
      expectedCandidate,
    );
    assert.equal(
      payload.rustStyleUnusedCanonicalProducer.boundedCheckerGate.includedInRustReleaseBundle,
      true,
      "release gate should be true",
    );
    assert.equal(payload.rustStyleUnusedConsistency.findingsMatch, true);
    assert.equal(payload.rustStyleUnusedConsistency.countsMatch, true);

    process.stdout.write(
      "== rust-checker-style-unused-consumer:stylelint-smoke-unused-selector ==\nvalidated code=unused-selector consistent=true releaseGate=true\n\n",
    );
  })().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}

async function run_rust_expression_domain_candidates(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-expression-domain-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainCandidatesInput(snapshot.input);

    assertExpressionDomainCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(`matched expression domain candidates: ${actual.candidates.length}\n\n`);
  }
}

async function run_rust_expression_domain_canonical_candidate(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-expression-domain-canonical-candidate:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainCanonicalCandidateBundle(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainCanonicalCandidateInput(snapshot.input);

    assertExpressionDomainCanonicalCandidateBundleMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated expression domain canonical-candidate bundle:",
        `planned=${actual.planSummary.plannedExpressionIds.length}`,
        `fragments=${actual.fragments.length}`,
        `candidates=${actual.candidates.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_expression_domain_canonical_producer(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_EXPRESSION_DOMAIN_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-domain",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-domain",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_EXPRESSION_DOMAIN_CORPUS) {
    process.stdout.write(`== rust-expression-domain-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainCanonicalProducerSignal(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainCanonicalProducerInput(snapshot.input);

    assertExpressionDomainCanonicalProducerSignalMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated expression domain canonical-producer signal:",
        `planned=${actual.canonicalBundle.planSummary.plannedExpressionIds.length}`,
        `fragments=${actual.canonicalBundle.fragments.length}`,
        `candidates=${actual.canonicalBundle.candidates.length}`,
        `evaluator=${actual.evaluatorCandidates.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_expression_domain_compare(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-expression-domain-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainPlanSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainInput(snapshot.input);

    assertExpressionDomainPlanSummaryMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched expression domain plan: expressions=${actual.plannedExpressionIds.length} finiteValues=${actual.finiteValueCount}\n\n`,
    );
  }
}

async function run_rust_expression_domain_evaluator_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_EXPRESSION_DOMAIN_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-domain",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-domain",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_EXPRESSION_DOMAIN_CORPUS) {
    process.stdout.write(`== rust-expression-domain-evaluator-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainEvaluatorCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainEvaluatorCandidatesInput(snapshot.input);

    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      throw new Error(
        [
          `${entry.label}: expression-domain evaluator candidates mismatch`,
          `actual=${JSON.stringify(actual, null, 2)}`,
          `expected=${JSON.stringify(expected, null, 2)}`,
        ].join("\n"),
      );
    }

    process.stdout.write(
      `matched expression domain evaluator candidates: ${actual.results.length}\n\n`,
    );
  }
}

async function run_rust_expression_domain_fragments(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-expression-domain-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionDomainFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionDomainFragmentsInput(snapshot.input);

    assertExpressionDomainFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(`matched expression domain fragments: ${actual.fragments.length}\n\n`);
  }
}

async function run_rust_expression_domain_reduced_evaluator(): Promise<void> {
  const INPUT: EngineInputV2 = {
    version: "2",
    workspace: {
      root: "/tmp/cme-expression-domain-reduced-evaluator",
      classnameTransform: "asIs",
      settingsKey: "synthetic-expression-domain-reduced-evaluator",
    },
    sources: [],
    styles: [],
    typeFacts: [
      fact("finite-prefix-exact", {
        kind: "finiteSet",
        values: ["btn-active", "card"],
        constraintKind: "prefix",
        prefix: "btn-",
      }),
      fact("finite-prefix-bottom", {
        kind: "finiteSet",
        values: ["card", "nav"],
        constraintKind: "prefix",
        prefix: "btn-",
      }),
      fact("constrained-prefix-values-finite", {
        kind: "constrained",
        values: ["btn-primary", "btn-secondary", "card"],
        constraintKind: "prefix",
        prefix: "btn-",
      }),
      fact("constrained-composite", {
        kind: "constrained",
        constraintKind: "composite",
        prefix: "btn-",
        suffix: "-active",
        minLen: 14,
        charMust: "-",
        charMay: "abcdefghijklmnopqrstuvwxyz-",
        mayIncludeOtherChars: false,
      }),
    ],
  };

  const EXPECTED_RAW_KINDS = new Map(
    INPUT.typeFacts.map((entry) => [entry.expressionId, entry.facts.kind]),
  );

  const EXPECTED_REDUCED_EVALUATOR_KINDS = new Map([
    ["finite-prefix-exact", "exact"],
    ["finite-prefix-bottom", "bottom"],
    ["constrained-prefix-values-finite", "finiteSet"],
    ["constrained-composite", "composite"],
  ]);

  process.stdout.write("== rust-expression-domain-reduced-evaluator:synthetic ==\n");

  const fragments = await runShadowExpressionDomainFragmentsInput(INPUT);
  const candidates = await runShadowExpressionDomainCandidatesInput(INPUT);
  const evaluatorCandidates = await runShadowExpressionDomainEvaluatorCandidatesInput(INPUT);

  for (const fragment of fragments.fragments) {
    assertEqual(
      fragment.valueDomainKind,
      EXPECTED_RAW_KINDS.get(fragment.expressionId),
      `${fragment.expressionId}: raw fragment must preserve input fact kind`,
    );
  }

  for (const candidate of candidates.candidates) {
    assertEqual(
      candidate.valueDomainKind,
      EXPECTED_RAW_KINDS.get(candidate.expressionId),
      `${candidate.expressionId}: raw candidate must preserve input fact kind`,
    );
  }

  for (const result of evaluatorCandidates.results) {
    assertEqual(
      result.payload.valueDomainKind,
      EXPECTED_REDUCED_EVALUATOR_KINDS.get(result.queryId),
      `${result.queryId}: evaluator candidate must expose reduced fact kind`,
    );
  }

  process.stdout.write(
    `validated reduced evaluator split: raw=${fragments.fragments.length} evaluator=${evaluatorCandidates.results.length}\n`,
  );

  function fact(expressionId: string, facts: StringTypeFactsV2) {
    return {
      filePath: "/tmp/App.tsx",
      expressionId,
      facts,
    };
  }

  function assertEqual(actual: unknown, expected: unknown, label: string): void {
    if (actual !== expected) {
      throw new Error(
        `${label}\nactual=${JSON.stringify(actual)}\nexpected=${JSON.stringify(expected)}`,
      );
    }
  }
}

async function run_rust_expression_semantics_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_EXPRESSION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-expression-semantics-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_EXPRESSION_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsCandidatesInput(snapshot.input);

    assertExpressionSemanticsCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched expression semantics candidates: ${actual.candidates.length}\n\n`,
    );
  }
}

async function run_rust_expression_semantics_canonical_candidate(): Promise<void> {
  const workspaceRoot = process.cwd();

  const EXPRESSION_SEMANTICS_CANONICAL_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-expression-semantics-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of EXPRESSION_SEMANTICS_CANONICAL_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-canonical-candidate:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsCanonicalCandidateBundle(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsCanonicalCandidateInput(snapshot.input);

    assertExpressionSemanticsCanonicalCandidateBundleMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated expression semantics canonical-candidate bundle:",
        `queries=${actual.queryFragments.length}`,
        `fragments=${actual.fragments.length}`,
        `matches=${actual.matchFragments.length}`,
        `candidates=${actual.candidates.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_expression_semantics_canonical_producer(): Promise<void> {
  const workspaceRoot = process.cwd();

  const EXPRESSION_SEMANTICS_CANONICAL_PRODUCER_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-expression-semantics-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of EXPRESSION_SEMANTICS_CANONICAL_PRODUCER_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsCanonicalProducerSignal(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsCanonicalProducerInput(snapshot.input);

    assertExpressionSemanticsCanonicalProducerSignalMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated expression semantics canonical-producer signal:",
        `queries=${actual.canonicalBundle.queryFragments.length}`,
        `fragments=${actual.canonicalBundle.fragments.length}`,
        `matches=${actual.canonicalBundle.matchFragments.length}`,
        `candidates=${actual.canonicalBundle.candidates.length}`,
        `evaluator=${actual.evaluatorCandidates.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_expression_semantics_evaluator_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const EXPRESSION_SEMANTICS_EVALUATOR_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-expression-semantics-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of EXPRESSION_SEMANTICS_EVALUATOR_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-evaluator-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsEvaluatorCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsEvaluatorCandidatesInput(snapshot.input);

    assertExpressionSemanticsEvaluatorCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched expression semantics evaluator candidates: ${actual.results.length}\n\n`,
    );
  }
}

async function run_rust_expression_semantics_fragments(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_EXPRESSION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_EXPRESSION_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsFragmentsInput(snapshot.input);

    assertExpressionSemanticsFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(`matched expression semantics fragments: ${actual.fragments.length}\n\n`);
  }
}

async function run_rust_expression_semantics_match_fragments(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_EXPRESSION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-expression-semantics-match",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-expression-semantics-match",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_EXPRESSION_CORPUS) {
    process.stdout.write(`== rust-expression-semantics-match-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsMatchFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsMatchFragmentsInput(snapshot.input);

    assertExpressionSemanticsMatchFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched expression semantics match fragments: ${actual.fragments.length}\n\n`,
    );
  }
}

async function run_rust_expression_semantics_query_fragments(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-expression-semantics-query-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsExpressionSemanticsQueryFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowExpressionSemanticsQueryFragmentsInput(snapshot.input);

    assertExpressionSemanticsQueryFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched expression semantics query fragments: ${actual.fragments.length}\n\n`,
    );
  }
}

async function run_rust_query_plan_compare(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-query-plan-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsQueryPlanSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowQueryPlanInput(snapshot.input);

    assertQueryPlanSummaryMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched query plan: expr=${actual.expressionSemanticsIds.length} selectorUsage=${actual.selectorUsageIds.length} total=${actual.totalQueryCount}\n\n`,
    );
  }
}

async function run_rust_selector_usage_fragments(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-selector-usage-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSelectorUsageFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSelectorUsageFragmentsInput(snapshot.input);

    assertSelectorUsageFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(`matched selector usage fragments: ${actual.fragments.length}\n\n`);
  }
}

async function run_rust_selector_usage_plan_compare(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-selector-usage-plan-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSelectorUsagePlanSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSelectorUsagePlanInput(snapshot.input);

    assertSelectorUsagePlanSummaryMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched selector usage plan: canonical=${actual.canonicalSelectorNames.length} composed=${actual.composedSelectorCount} composesRefs=${actual.totalComposesRefs}\n\n`,
    );
  }
}

async function run_rust_selector_usage_query_fragments(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-selector-usage-query-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSelectorUsageQueryFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSelectorUsageQueryFragmentsInput(snapshot.input);

    assertSelectorUsageQueryFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(`matched selector usage query fragments: ${actual.fragments.length}\n\n`);
  }
}

async function run_rust_semantic_canonical_candidate(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SEMANTIC_CANONICAL_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-semantic-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-semantic-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-semantic-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SEMANTIC_CANONICAL_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-semantic-canonical-candidate:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSemanticCanonicalCandidateBundle(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSemanticCanonicalCandidateInput(snapshot.input);

    assertSemanticCanonicalCandidateBundleMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated semantic canonical-candidate bundle:",
        `expressionCandidates=${actual.sourceSide.expressionSemantics.candidates.length}`,
        `resolutionCandidates=${actual.sourceSide.sourceResolution.candidates.length}`,
        `domainCandidates=${actual.expressionDomain.candidates.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_semantic_canonical_producer(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SEMANTIC_CANONICAL_PRODUCER_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-semantic-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-semantic-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-semantic-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SEMANTIC_CANONICAL_PRODUCER_CORPUS) {
    process.stdout.write(`== rust-semantic-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSemanticCanonicalProducerSignal(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSemanticCanonicalProducerInput(snapshot.input);

    assertSemanticCanonicalProducerSignalMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated semantic canonical-producer signal:",
        `expressionCandidates=${actual.canonicalBundle.sourceSide.expressionSemantics.candidates.length}`,
        `resolutionCandidates=${actual.canonicalBundle.sourceSide.sourceResolution.candidates.length}`,
        `domainCandidates=${actual.canonicalBundle.expressionDomain.candidates.length}`,
        `expressionEvaluator=${actual.evaluatorCandidates.sourceSide.expressionSemantics.results.length}`,
        `resolutionEvaluator=${actual.evaluatorCandidates.sourceSide.sourceResolution.results.length}`,
        `domainEvaluator=${actual.evaluatorCandidates.expressionDomain.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_semantic_evaluator_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SEMANTIC_EVALUATOR_CANDIDATES_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-semantic-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-semantic-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-semantic-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SEMANTIC_EVALUATOR_CANDIDATES_CORPUS) {
    process.stdout.write(`== rust-semantic-evaluator-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSemanticEvaluatorCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSemanticEvaluatorCandidatesInput(snapshot.input);

    assertSemanticEvaluatorCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated semantic evaluator candidates:",
        `expressionEvaluator=${actual.sourceSide.expressionSemantics.results.length}`,
        `resolutionEvaluator=${actual.sourceSide.sourceResolution.results.length}`,
        `domainEvaluator=${actual.expressionDomain.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_shadow_compare(): Promise<void> {
  const OVERLAP_PARITY_LABEL = "source-prefix-suffix-overlap-parity-v2";

  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-shadow-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsShadowSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadow(snapshot);

    assertShadowSummaryMatch(entry.label, actual, expected);

    if (entry.authoredMembership) {
      if (entry.label === OVERLAP_PARITY_LABEL) {
        // oxlint-disable-next-line eslint/no-await-in-loop
        await assertRustOverlapDefaults(snapshot.input, entry.authoredMembership);
      }
      // oxlint-disable-next-line eslint/no-await-in-loop
      const membership = await runShadowSourceResolutionCandidatesInput(
        authoredMembershipInput(snapshot.input, entry.authoredMembership),
      );
      assert.equal(
        membership.candidates.length,
        1,
        `${entry.label}: authored membership must reach exactly one Rust source candidate`,
      );
      assert.deepEqual(
        membership.candidates[0]?.selectorNames,
        entry.authoredMembership.selectorNames,
        `${entry.label}: Rust source-resolution membership must match the authored selector set`,
      );
    }

    process.stdout.write(
      `matched summary fields: sources=${actual.sourceCount} styles=${actual.styleCount} typeFacts=${actual.typeFactCount} queries=${actual.queryResultCount} findings=${actual.checkerTotalFindings}\n\n`,
    );
  }

  function authoredMembershipInput(
    input: EngineInputV2,
    expected: ContractParityAuthoredMembershipV2,
  ): EngineInputV2 {
    assert.equal(input.typeFacts.length, 1, "authored membership requires one type-fact entry");
    return {
      ...input,
      typeFacts: input.typeFacts.map(({ controlFlowGraph: _controlFlowGraph, ...entry }) => ({
        ...entry,
        facts: {
          kind: "constrained",
          constraintKind: "prefixSuffix",
          prefix: expected.valuePrefix,
          suffix: expected.valueSuffix,
          minLen: expected.valueMinLen,
        },
      })),
    };
  }

  async function assertRustOverlapDefaults(
    input: EngineInputV2,
    expected: ContractParityAuthoredMembershipV2,
  ): Promise<void> {
    const prefixSuffix = await runShadowExpressionDomainEvaluatorCandidatesInput(
      authoredOverlapDefaultInput(input, expected, "prefixSuffix"),
    );
    assertRustOverlapDefaultValue(prefixSuffix, "prefixSuffix", expected);

    const composite = await runShadowExpressionDomainEvaluatorCandidatesInput(
      authoredOverlapDefaultInput(input, expected, "composite"),
    );
    assertRustOverlapDefaultValue(composite, "composite", expected);
  }

  function authoredOverlapDefaultInput(
    input: EngineInputV2,
    expected: ContractParityAuthoredMembershipV2,
    constraintKind: "prefixSuffix" | "composite",
  ): EngineInputV2 {
    const edgeChars = Array.from(new Set(Array.from(expected.valuePrefix + expected.valueSuffix)))
      .toSorted()
      .join("");
    return {
      ...input,
      typeFacts: input.typeFacts.map(({ controlFlowGraph: _controlFlowGraph, ...entry }) => ({
        ...entry,
        facts: {
          kind: "constrained",
          constraintKind,
          prefix: expected.valuePrefix,
          suffix: expected.valueSuffix,
          ...(constraintKind === "composite" ? { charMust: edgeChars, charMay: edgeChars } : {}),
        },
      })),
    };
  }

  function assertRustOverlapDefaultValue(
    candidates: Awaited<ReturnType<typeof runShadowExpressionDomainEvaluatorCandidatesInput>>,
    expectedKind: "prefixSuffix" | "composite",
    expected: ContractParityAuthoredMembershipV2,
  ): void {
    assert.equal(
      candidates.results.length,
      1,
      `${expectedKind}: Rust default probe candidate count`,
    );
    const value = candidates.results[0]?.payload.valueDomainProvenanceTree.value as
      | Readonly<Record<string, unknown>>
      | undefined;
    assert.deepEqual(
      {
        kind: value?.kind,
        prefix: value?.prefix,
        suffix: value?.suffix,
        minLength: value?.minLength,
      },
      {
        kind: expectedKind,
        prefix: expected.valuePrefix,
        suffix: expected.valueSuffix,
        minLength: expected.valueMinLen,
      },
      `${expectedKind}: Rust byte-domain default overlap must match the authored ASCII minimum`,
    );
  }
}

async function run_rust_shadow_smoke(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-shadow:${entry.label} ==\n`);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const summary = await runShadow(snapshot);
    const expected = deriveTsShadowSummary(snapshot);
    assertShadowSummaryMatch(entry.label, summary, expected);

    process.stdout.write(
      `sources=${summary.sourceCount} styles=${summary.styleCount} typeFacts=${summary.typeFactCount} queries=${summary.queryResultCount} findings=${summary.checkerTotalFindings} kinds=${JSON.stringify(summary.byKind)} queryKinds=${JSON.stringify(summary.queryKindCounts)}\n\n`,
    );
  }
}

async function run_rust_source_resolution_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-resolution-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS) {
    process.stdout.write(`== rust-source-resolution-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionCandidatesInput(snapshot.input);

    assertSourceResolutionCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(`matched source resolution candidates: ${actual.candidates.length}\n\n`);
  }
}

async function run_rust_source_resolution_canonical_candidate(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_RESOLUTION_CANONICAL_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-resolution-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_RESOLUTION_CANONICAL_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-source-resolution-canonical-candidate:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionCanonicalCandidateBundle(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionCanonicalCandidateInput(snapshot.input);

    assertSourceResolutionCanonicalCandidateBundleMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated source resolution canonical-candidate bundle:",
        `queries=${actual.queryFragments.length}`,
        `fragments=${actual.fragments.length}`,
        `matches=${actual.matchFragments.length}`,
        `candidates=${actual.candidates.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_source_resolution_canonical_producer(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_RESOLUTION_CANONICAL_PRODUCER_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-resolution-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_RESOLUTION_CANONICAL_PRODUCER_CORPUS) {
    process.stdout.write(`== rust-source-resolution-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionCanonicalProducerSignal(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionCanonicalProducerInput(snapshot.input);

    assertSourceResolutionCanonicalProducerSignalMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated source resolution canonical-producer signal:",
        `queries=${actual.canonicalBundle.queryFragments.length}`,
        `fragments=${actual.canonicalBundle.fragments.length}`,
        `matches=${actual.canonicalBundle.matchFragments.length}`,
        `candidates=${actual.canonicalBundle.candidates.length}`,
        `evaluator=${actual.evaluatorCandidates.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_source_resolution_evaluator_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_RESOLUTION_EVALUATOR_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-resolution-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_RESOLUTION_EVALUATOR_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-source-resolution-evaluator-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionEvaluatorCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionEvaluatorCandidatesInput(snapshot.input);

    assertSourceResolutionEvaluatorCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched source resolution evaluator candidates: ${actual.results.length}\n\n`,
    );
  }
}

async function run_rust_source_resolution_fragments(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS) {
    process.stdout.write(`== rust-source-resolution-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionFragmentsInput(snapshot.input);

    assertSourceResolutionFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(`matched source resolution fragments: ${actual.fragments.length}\n\n`);
  }
}

async function run_rust_source_resolution_match_fragments(): Promise<void> {
  const workspaceRoot = process.cwd();

  const TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-resolution-match",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-resolution-match",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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

  for (const entry of TYPE_FACT_BACKED_SOURCE_RESOLUTION_CORPUS) {
    process.stdout.write(`== rust-source-resolution-match-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionMatchFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionMatchFragmentsInput(snapshot.input);

    assertSourceResolutionMatchFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched source resolution match fragments: ${actual.fragments.length}\n\n`,
    );
  }
}

async function run_rust_source_resolution_plan_compare(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-source-resolution-plan-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionPlanSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionPlanInput(snapshot.input);

    assertSourceResolutionPlanSummaryMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched source resolution plan: expressions=${actual.plannedExpressionIds.length} styles=${actual.distinctStyleFilePaths.length} styleAccess=${actual.styleAccessCount}\n\n`,
    );
  }
}

async function run_rust_source_resolution_query_fragments(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-source-resolution-query-fragments:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceResolutionQueryFragments(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceResolutionQueryFragmentsInput(snapshot.input);

    assertSourceResolutionQueryFragmentsMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched source resolution query fragments: ${actual.fragments.length}\n\n`,
    );
  }
}

async function run_rust_source_side_canonical_candidate(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_SIDE_CANONICAL_CANDIDATE_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-side-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-side-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-side-canonical-candidate",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_SIDE_CANONICAL_CANDIDATE_CORPUS) {
    process.stdout.write(`== rust-source-side-canonical-candidate:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceSideCanonicalCandidateBundle(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceSideCanonicalCandidateInput(snapshot.input);

    assertSourceSideCanonicalCandidateBundleMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated source-side canonical-candidate bundle:",
        `expressionCandidates=${actual.expressionSemantics.candidates.length}`,
        `resolutionCandidates=${actual.sourceResolution.candidates.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_source_side_canonical_producer(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_SIDE_CANONICAL_PRODUCER_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-side-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-side-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-side-canonical-producer",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_SIDE_CANONICAL_PRODUCER_CORPUS) {
    process.stdout.write(`== rust-source-side-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceSideCanonicalProducerSignal(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceSideCanonicalProducerInput(snapshot.input);

    assertSourceSideCanonicalProducerSignalMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated source-side canonical-producer signal:",
        `expressionCandidates=${actual.canonicalBundle.expressionSemantics.candidates.length}`,
        `expressionEvaluator=${actual.evaluatorCandidates.expressionSemantics.results.length}`,
        `resolutionCandidates=${actual.canonicalBundle.sourceResolution.candidates.length}`,
        `resolutionEvaluator=${actual.evaluatorCandidates.sourceResolution.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_source_side_evaluator_candidates(): Promise<void> {
  const workspaceRoot = process.cwd();

  const SOURCE_SIDE_EVALUATOR_CANDIDATES_CORPUS: readonly ContractParityEntry[] = [
    {
      label: "literal-union-source-side-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
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
      label: "path-alias-source-side-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/path-alias",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss",
          ),
        ],
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
      label: "composite-source-side-evaluator-candidates",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
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

  for (const entry of SOURCE_SIDE_EVALUATOR_CANDIDATES_CORPUS) {
    process.stdout.write(`== rust-source-side-evaluator-candidates:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsSourceSideEvaluatorCandidates(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowSourceSideEvaluatorCandidatesInput(snapshot.input);

    assertSourceSideEvaluatorCandidatesMatch(entry.label, actual, expected);

    process.stdout.write(
      [
        "validated source-side evaluator candidates:",
        `expressionEvaluator=${actual.expressionSemantics.results.length}`,
        `resolutionEvaluator=${actual.sourceResolution.results.length}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
}

async function run_rust_type_fact_compare(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-type-fact-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsTypeFactInputSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadowTypeFactInput(snapshot.input);

    assertTypeFactSummaryMatch(entry.label, actual, expected);

    process.stdout.write(
      `matched type-fact fields: count=${actual.typeFactCount} distinctFiles=${actual.distinctFactFiles} finiteValues=${actual.finiteValueCount} kinds=${JSON.stringify(actual.byKind)}\n\n`,
    );
  }

  function assertTypeFactSummaryMatch(
    label: string,
    actual: TypeFactInputSummaryV0,
    expected: TypeFactInputSummaryV0,
  ) {
    assertEqual(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
    assertEqual(label, "inputVersion", actual.inputVersion, expected.inputVersion);
    assertEqual(label, "typeFactCount", actual.typeFactCount, expected.typeFactCount);
    assertEqual(label, "distinctFactFiles", actual.distinctFactFiles, expected.distinctFactFiles);
    assertEqual(label, "finiteValueCount", actual.finiteValueCount, expected.finiteValueCount);
    assertJsonEqual(label, "byKind", actual.byKind, expected.byKind);
    assertJsonEqual(label, "constrainedKinds", actual.constrainedKinds, expected.constrainedKinds);
  }

  function assertEqual<T>(label: string, field: string, actual: T, expected: T) {
    if (actual !== expected) {
      throw new Error(
        `${label}: ${field} mismatch\nexpected: ${JSON.stringify(expected)}\nreceived: ${JSON.stringify(actual)}`,
      );
    }
  }

  function assertJsonEqual(label: string, field: string, actual: unknown, expected: unknown) {
    const actualJson = JSON.stringify(actual);
    const expectedJson = JSON.stringify(expected);
    if (actualJson !== expectedJson) {
      throw new Error(
        `${label}: ${field} mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
      );
    }
  }
}

export const RUST_SHADOW_FAMILY: {
  readonly [slug: string]: { readonly corpus: "shared" | "own"; readonly run: () => Promise<void> };
} = {
  "rust-checker-source-missing-canonical-candidate": {
    corpus: "own",
    run: run_rust_checker_source_missing_canonical_candidate,
  },
  "rust-checker-source-missing-canonical-producer": {
    corpus: "own",
    run: run_rust_checker_source_missing_canonical_producer,
  },
  "rust-checker-style-recovery-canonical-candidate": {
    corpus: "own",
    run: run_rust_checker_style_recovery_canonical_candidate,
  },
  "rust-checker-style-recovery-canonical-producer": {
    corpus: "own",
    run: run_rust_checker_style_recovery_canonical_producer,
  },
  "rust-checker-style-unused-canonical-candidate": {
    corpus: "own",
    run: run_rust_checker_style_unused_canonical_candidate,
  },
  "rust-checker-style-unused-canonical-producer": {
    corpus: "own",
    run: run_rust_checker_style_unused_canonical_producer,
  },
  "rust-checker-style-unused-consumer-boundary": {
    corpus: "own",
    run: run_rust_checker_style_unused_consumer_boundary,
  },
  "rust-expression-domain-candidates": {
    corpus: "shared",
    run: run_rust_expression_domain_candidates,
  },
  "rust-expression-domain-canonical-candidate": {
    corpus: "shared",
    run: run_rust_expression_domain_canonical_candidate,
  },
  "rust-expression-domain-canonical-producer": {
    corpus: "own",
    run: run_rust_expression_domain_canonical_producer,
  },
  "rust-expression-domain-compare": { corpus: "shared", run: run_rust_expression_domain_compare },
  "rust-expression-domain-evaluator-candidates": {
    corpus: "own",
    run: run_rust_expression_domain_evaluator_candidates,
  },
  "rust-expression-domain-fragments": {
    corpus: "shared",
    run: run_rust_expression_domain_fragments,
  },
  "rust-expression-domain-reduced-evaluator": {
    corpus: "own",
    run: run_rust_expression_domain_reduced_evaluator,
  },
  "rust-expression-semantics-candidates": {
    corpus: "own",
    run: run_rust_expression_semantics_candidates,
  },
  "rust-expression-semantics-canonical-candidate": {
    corpus: "own",
    run: run_rust_expression_semantics_canonical_candidate,
  },
  "rust-expression-semantics-canonical-producer": {
    corpus: "own",
    run: run_rust_expression_semantics_canonical_producer,
  },
  "rust-expression-semantics-evaluator-candidates": {
    corpus: "own",
    run: run_rust_expression_semantics_evaluator_candidates,
  },
  "rust-expression-semantics-fragments": {
    corpus: "own",
    run: run_rust_expression_semantics_fragments,
  },
  "rust-expression-semantics-match-fragments": {
    corpus: "own",
    run: run_rust_expression_semantics_match_fragments,
  },
  "rust-expression-semantics-query-fragments": {
    corpus: "shared",
    run: run_rust_expression_semantics_query_fragments,
  },
  "rust-query-plan-compare": { corpus: "shared", run: run_rust_query_plan_compare },
  "rust-selector-usage-fragments": { corpus: "shared", run: run_rust_selector_usage_fragments },
  "rust-selector-usage-plan-compare": {
    corpus: "shared",
    run: run_rust_selector_usage_plan_compare,
  },
  "rust-selector-usage-query-fragments": {
    corpus: "shared",
    run: run_rust_selector_usage_query_fragments,
  },
  "rust-semantic-canonical-candidate": {
    corpus: "own",
    run: run_rust_semantic_canonical_candidate,
  },
  "rust-semantic-canonical-producer": { corpus: "own", run: run_rust_semantic_canonical_producer },
  "rust-semantic-evaluator-candidates": {
    corpus: "own",
    run: run_rust_semantic_evaluator_candidates,
  },
  "rust-shadow-compare": { corpus: "shared", run: run_rust_shadow_compare },
  "rust-shadow-smoke": { corpus: "shared", run: run_rust_shadow_smoke },
  "rust-source-resolution-candidates": {
    corpus: "own",
    run: run_rust_source_resolution_candidates,
  },
  "rust-source-resolution-canonical-candidate": {
    corpus: "own",
    run: run_rust_source_resolution_canonical_candidate,
  },
  "rust-source-resolution-canonical-producer": {
    corpus: "own",
    run: run_rust_source_resolution_canonical_producer,
  },
  "rust-source-resolution-evaluator-candidates": {
    corpus: "own",
    run: run_rust_source_resolution_evaluator_candidates,
  },
  "rust-source-resolution-fragments": { corpus: "own", run: run_rust_source_resolution_fragments },
  "rust-source-resolution-match-fragments": {
    corpus: "own",
    run: run_rust_source_resolution_match_fragments,
  },
  "rust-source-resolution-plan-compare": {
    corpus: "shared",
    run: run_rust_source_resolution_plan_compare,
  },
  "rust-source-resolution-query-fragments": {
    corpus: "shared",
    run: run_rust_source_resolution_query_fragments,
  },
  "rust-source-side-canonical-candidate": {
    corpus: "own",
    run: run_rust_source_side_canonical_candidate,
  },
  "rust-source-side-canonical-producer": {
    corpus: "own",
    run: run_rust_source_side_canonical_producer,
  },
  "rust-source-side-evaluator-candidates": {
    corpus: "own",
    run: run_rust_source_side_evaluator_candidates,
  },
  "rust-type-fact-compare": { corpus: "shared", run: run_rust_type_fact_compare },
};
