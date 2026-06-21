import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  usesRustExpressionSemanticsBackend,
  usesRustSelectorUsageBackend,
  usesRustSourceResolutionBackend,
  usesRustStyleSemanticGraphBackend,
  type SelectedQueryBackendKind,
} from "../server/engine-host-node/src/selected-query-backend";
import { runShadowOmenaQuerySelectedQueryAdapterCapabilities } from "./rust-shadow-shared";

const BACKEND_KINDS: readonly SelectedQueryBackendKind[] = [
  "typescript-current",
  "rust-source-resolution",
  "rust-expression-semantics",
  "rust-selector-usage",
  "rust-selected-query",
];

interface StaticStylesheetEvaluatorOracleCorpusSummaryV0 {
  readonly product: string;
  readonly mode: string;
  readonly valueType: string;
  readonly productOutputSource: string;
  readonly legacyOutputRetainedAsOracleCount: number;
  readonly legacyOutputConsumedUntilCutoverCount: number;
  readonly allLegacyOutputsRetainedAsOracle: boolean;
  readonly fixtureCount: number;
  readonly scssFixtureCount: number;
  readonly sassFixtureCount: number;
  readonly lessFixtureCount: number;
  readonly evaluatedFixtureCount: number;
  readonly missingEvaluationCount: number;
  readonly divergenceCount: number;
  readonly nativeEditOutputMatchCount: number;
  readonly nativeValueEditCount: number;
  readonly nativeStructuralEditCount: number;
  readonly scssControlFlowValueTruthinessCount: number;
  readonly scssControlFlowContextualTruthinessFallbackCount: number;
  readonly scssControlFlowContextualTruthinessConflictCount: number;
  readonly scssControlFlowPruneReachabilityFixtureCount: number;
  readonly scssControlFlowPruneReachabilityChangedFixtureCount: number;
  readonly scssControlFlowPruneReachabilityFlatCssCfgBuiltCount: number;
  readonly nativeRawValueCount: number;
  readonly nativeTopValueCount: number;
  readonly nativeCycleValueCount: number;
  readonly nativeFuelExhaustedValueCount: number;
  readonly nativeUnresolvedReferenceValueCount: number;
  readonly nativeUnsupportedDynamicValueCount: number;
  readonly allLegacyDeclarationValuesPreserved: boolean;
  readonly allNativeEditOutputsMatchEvaluatedCss: boolean;
  readonly nativeProductOutputCorpusReady: boolean;
  readonly corpus?: StaticStylesheetEvaluatorOracleCorpusSummaryV0 & {
    readonly fixtures: readonly StaticStylesheetEvaluatorOracleFixtureSummaryV0[];
  };
}

interface StaticStylesheetEvaluatorOracleFixtureSummaryV0 {
  readonly id: string;
  readonly dialect: string;
  readonly productOutputSource: string;
  readonly legacyOutputRetainedAsOracle: boolean;
  readonly legacyOutputConsumedUntilCutover: boolean;
  readonly evaluationAvailable: boolean;
  readonly nativeEditOutput?: string;
  readonly divergenceCount: number;
  readonly nativeEditOutputMatchesEvaluatedCss: boolean;
  readonly nativeRawValueCount: number;
  readonly nativeTopValueCount: number;
  readonly nativeCycleValueCount: number;
  readonly nativeFuelExhaustedValueCount: number;
  readonly nativeUnresolvedReferenceValueCount: number;
  readonly nativeUnsupportedDynamicValueCount: number;
  readonly scssControlFlowValueTruthinessCount: number;
  readonly scssControlFlowContextualTruthinessFallbackCount: number;
  readonly scssControlFlowContextualTruthinessConflictCount: number;
  readonly scssControlFlowPruneReachabilityAvailable: boolean;
  readonly scssControlFlowPruneReachabilityConverged: boolean;
  readonly scssControlFlowPruneReachabilityFlatCssCfgBuilt: boolean;
  readonly scssControlFlowPruneReachabilityHaveTerminalsChanged: boolean;
  readonly scssControlFlowPruneReachabilityReachableBlockCount: number;
  readonly scssControlFlowPruneReachabilityUnreachableBlockCount: number;
}

interface StaticStylesheetEvaluatorSummaryV0 {
  readonly product: string;
  readonly mode: string;
  readonly dialect: string;
  readonly valueType: string;
  readonly supportedDialect: boolean;
  readonly productOutputSource: string;
  readonly legacyOutputRetainedAsOracle: boolean;
  readonly legacyOutputConsumedUntilCutover: boolean;
  readonly evaluationAvailable: boolean;
  readonly valueResolutionAvailable: boolean;
  readonly nativeEditOutput?: string;
  readonly divergenceCount: number;
  readonly allLegacyDeclarationValuesPreserved: boolean;
  readonly nativeEditOutputMatchesEvaluatedCss: boolean;
  readonly nativeValueReferenceCount: number;
  readonly nativeResolvedValueCount: number;
}

interface StaticLifExportsSummaryV0 {
  readonly product: string;
  readonly mode: string;
  readonly dialect: string;
  readonly sourceSyntax: string;
  readonly sifSuperset: boolean;
  readonly lessSpecificExportCount: number;
  readonly lessVariableCount: number;
  readonly lessMixinCount: number;
  readonly lessDetachedRulesetCount: number;
  readonly lessVariableNames: readonly string[];
  readonly lessMixinNames: readonly string[];
  readonly lessDetachedRulesetNames: readonly string[];
  readonly sifVariableCount: number;
  readonly exports: {
    readonly lessVariables: readonly {
      readonly name: string;
      readonly valueRepr?: string;
    }[];
    readonly lessMixins: readonly {
      readonly name: string;
      readonly guarded: boolean;
    }[];
    readonly lessDetachedRulesets: readonly {
      readonly name: string;
      readonly memberNames: readonly string[];
    }[];
  };
}

interface CrossFileSummaryV0 {
  readonly product: string;
  readonly status: string;
  readonly summaryScope: string;
  readonly styleCount: number;
  readonly summaryEdgeCount: number;
  readonly edgeKindCounts: readonly {
    readonly edgeKind: string;
    readonly count: number;
  }[];
  readonly edges: readonly {
    readonly edgeKind: string;
    readonly fromKind: string;
    readonly fromPath: string;
    readonly targetKind?: string;
    readonly targetPath?: string;
    readonly source?: string;
    readonly status: string;
    readonly provenance: readonly string[];
  }[];
  readonly capabilities: {
    readonly sassModuleEdgesReady: boolean;
    readonly stableSummaryHashReady: boolean;
    readonly linearProvenanceReady: boolean;
    readonly linearProvenanceRoundTripReady: boolean;
  };
}

interface ScssEvaluatorControlFlowOracleCorpusSummaryV0 {
  readonly product: string;
  readonly mode: string;
  readonly valueType: string;
  readonly nodeKeyType: string;
  readonly recursionCap: number;
  readonly fixtureCount: number;
  readonly scssFixtureCount: number;
  readonly sassFixtureCount: number;
  readonly supportedFixtureCount: number;
  readonly rejectedFlatCssFixtureCount: number;
  readonly controlFlowFixtureCount: number;
  readonly branchFixtureCount: number;
  readonly loopFixtureCount: number;
  readonly backEdgeFixtureCount: number;
  readonly callReturnFixtureCount: number;
  readonly resolvedCallReturnFixtureCount: number;
  readonly topCallReturnFixtureCount: number;
  readonly recursiveCallFixtureCount: number;
  readonly convergedValueAnalysisFixtureCount: number;
  readonly widenedToTopFixtureCount: number;
  readonly wideningWitnessWidenedToTopCount: number;
  readonly wideningWitnessConverged: boolean;
  readonly pruneReachabilityFixtureCount: number;
  readonly pruneReachabilityChangedFixtureCount: number;
  readonly pruneReachabilityFlatCssCfgBuiltCount: number;
  readonly flatCssCfgBuiltCount: number;
  readonly mergedCrossFileGraphCount: number;
  readonly allSupportedFixturesConverged: boolean;
  readonly noFlatCssCfgBuilt: boolean;
  readonly noMergedCrossFileGraph: boolean;
  readonly wideningWitness: ScssEvaluatorControlFlowWideningWitnessV0;
  readonly corpus?: ScssEvaluatorControlFlowOracleCorpusSummaryV0 & {
    readonly fixtures: readonly ScssEvaluatorControlFlowOracleFixtureSummaryV0[];
  };
}

interface ScssEvaluatorControlFlowSummaryV0 {
  readonly product: string;
  readonly mode: string;
  readonly dialect: string;
  readonly valueType: string;
  readonly supportedDialect: boolean;
  readonly flatCssCfgBuilt: boolean;
  readonly mergedCrossFileGraph: boolean;
  readonly controlFlowBlockCount: number;
  readonly controlFlowBranchBlockCount: number;
  readonly callReturnNodeCount: number;
  readonly valueAnalysisConverged: boolean;
  readonly pruneReachabilityAvailable: boolean;
  readonly pruneReachabilityConverged: boolean;
  readonly pruneReachabilityFlatCssCfgBuilt: boolean;
  readonly readySurfaces: readonly string[];
}

interface ScssEvaluatorControlFlowWideningWitnessV0 {
  readonly product: string;
  readonly mode: string;
  readonly valueType: string;
  readonly policy: string;
  readonly maxIterations: number;
  readonly nodeCount: number;
  readonly converged: boolean;
  readonly iterationCount: number;
  readonly widenedToTopCount: number;
  readonly outputTopCount: number;
}

interface ScssEvaluatorControlFlowOracleFixtureSummaryV0 {
  readonly id: string;
  readonly dialect: string;
  readonly supportedDialect: boolean;
  readonly controlFlowAvailable: boolean;
  readonly valueAnalysisAvailable: boolean;
  readonly callReturnAvailable: boolean;
  readonly branchBlockCount: number;
  readonly loopBlockCount: number;
  readonly backEdgeCount: number;
  readonly callResolvedReturnValueCount: number;
  readonly exactCallResolvedReturnValueCount: number;
  readonly topCallResolvedReturnValueCount: number;
  readonly recursiveEdgeCount: number;
  readonly cappedRecursiveCallCount: number;
  readonly valueAnalysisConverged: boolean;
  readonly valueAnalysisIterationCount: number;
  readonly widenedToTopCount: number;
  readonly flatCssCfgBuilt: boolean;
  readonly mergedCrossFileGraph: boolean;
}

const EXPECTED_RUNNER_COMMANDS = new Map([
  [
    "queryEvaluationRuntime",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.queryEvaluationRuntime,
      inputContract: "EngineInputV2 + OmenaQueryExpressionDomainFlowRuntimeV0",
      outputProduct: "omena-query.evaluation-runtime",
    },
  ],
  [
    "sourceResolution",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.sourceResolutionCanonicalProducer,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.source-resolution-canonical-producer",
    },
  ],
  [
    "sourceResolutionRuntime",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.sourceResolutionRuntime,
      inputContract: "EngineInputV2",
      outputProduct: "omena-resolver.source-resolution-runtime-index",
    },
  ],
  [
    "expressionSemantics",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionSemanticsCanonicalProducer,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-semantics-canonical-producer",
    },
  ],
  [
    "expressionDomainFlowAnalysis",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainFlowAnalysis,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-domain-flow-analysis",
    },
  ],
  [
    "expressionDomainControlFlowAnalysis",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainControlFlowAnalysis,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-domain-control-flow-analysis",
    },
  ],
  [
    "expressionDomainCallSiteFlowAnalysis",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainCallSiteFlowAnalysis,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-domain-call-site-flow-analysis",
    },
  ],
  [
    "expressionDomainProvenanceExplanations",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainProvenanceExplanations,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-domain-provenance-explanations",
    },
  ],
  [
    "expressionDomainReducedProductIteration",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainReducedProductIteration,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.expression-domain-reduced-product-iteration",
    },
  ],
  [
    "expressionDomainIncrementalFlowAnalysis",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainIncrementalFlowAnalysis,
      inputContract: "EngineInputV2 + OmenaQueryExpressionDomainFlowRuntimeV0",
      outputProduct: "omena-query.expression-domain-incremental-flow-analysis",
    },
  ],
  [
    "expressionDomainSelectorProjection",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainSelectorProjection,
      inputContract: "EngineInputV2",
      outputProduct: "omena-query.expression-domain-selector-projection",
    },
  ],
  [
    "scssEvaluatorControlFlow",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.scssEvaluatorControlFlow,
      inputContract: "EngineInputV2 + targetStylePath",
      outputProduct: "omena-query.scss-evaluator-control-flow",
    },
  ],
  [
    "scssEvaluatorControlFlowOracleCorpus",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.scssEvaluatorControlFlowOracleCorpus,
      inputContract: "none",
      outputProduct: "omena-query.scss-evaluator-control-flow-oracle-corpus",
    },
  ],
  [
    "staticStylesheetEvaluator",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.staticStylesheetEvaluator,
      inputContract: "EngineInputV2 + targetStylePath",
      outputProduct: "omena-query.static-stylesheet-evaluator",
    },
  ],
  [
    "staticStylesheetEvaluatorOracleCorpus",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.staticStylesheetEvaluatorOracleCorpus,
      inputContract: "none",
      outputProduct: "omena-query.static-stylesheet-evaluator-oracle-corpus",
    },
  ],
  [
    "staticLifExports",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.staticLifExports,
      inputContract: "EngineInputV2 + targetStylePath",
      outputProduct: "omena-query.static-lif-exports",
    },
  ],
  [
    "selectorUsage",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.selectorUsageCanonicalProducer,
      inputContract: "EngineInputV2",
      outputProduct: "engine-input-producers.selector-usage-canonical-producer",
    },
  ],
  [
    "omenaParserStyleFacts",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.omenaParserStyleFacts,
      inputContract: "OmenaParserStyleFactsInputV0",
      outputProduct: "omena-parser.style-facts",
    },
  ],
  [
    "styleSemanticGraph",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraph,
      inputContract: "StyleSemanticGraphInputV0",
      outputProduct: "omena-semantic.style-semantic-graph",
    },
  ],
  [
    "readCascadeAtPosition",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.readCascadeAtPosition,
      inputContract: "ReadCascadeAtPositionInputV0",
      outputProduct: "omena-query.read-cascade-at-position",
    },
  ],
  [
    "styleDiagnosticsForFile",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.styleDiagnosticsForFile,
      inputContract: "StyleDiagnosticsForFileInputV0",
      outputProduct: "omena-query.diagnostics-for-file",
    },
  ],
  [
    "sourceDiagnosticsForFile",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile,
      inputContract: "SourceDiagnosticsForFileInputV0",
      outputProduct: "omena-query.diagnostics-for-file",
    },
  ],
  [
    "completionAt",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.completionAt,
      inputContract: "CompletionAtInputV0",
      outputProduct: "omena-query.completion-at",
    },
  ],
  [
    "styleCodeActions",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.styleCodeActions,
      inputContract: "StyleCodeActionsInputV0",
      outputProduct: "omena-query.code-actions",
    },
  ],
  [
    "refsForClass",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.refsForClass,
      inputContract: "RefsForClassInputV0",
      outputProduct: "omena-query.refs-for-class",
    },
  ],
  [
    "renamePlan",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.renamePlan,
      inputContract: "RenamePlanInputV0",
      outputProduct: "omena-query.rename-plan",
    },
  ],
  [
    "readStyleContextIndex",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.readStyleContextIndex,
      inputContract: "ReadStyleContextIndexInputV0",
      outputProduct: "omena-query.style-context-index",
    },
  ],
  [
    "styleSemanticGraphBatch",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraphBatch,
      inputContract: "StyleSemanticGraphBatchInputV0",
      outputProduct: "omena-semantic.style-semantic-graph-batch",
    },
  ],
  [
    "transformPlan",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.transformPlan,
      inputContract: "TransformPlanInputV0",
      outputProduct: "omena-query.transform-plan",
    },
  ],
  [
    "transformContext",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.transformContext,
      inputContract: "TransformContextInputV0",
      outputProduct: "omena-query.transform-context",
    },
  ],
  [
    "semanticReachabilityTransformContext",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.semanticReachabilityTransformContext,
      inputContract: "EngineInputV2 + targetStylePath + closedStyleWorld",
      outputProduct: "omena-query.transform-context-from-engine-input",
    },
  ],
  [
    "transformExecute",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.transformExecute,
      inputContract: "TransformExecuteInputV0",
      outputProduct: "omena-query.transform-execute",
    },
  ],
  [
    "consumerCheckStyleSource",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.consumerCheckStyleSource,
      inputContract: "ConsumerStyleSourceInputV0",
      outputProduct: "omena-query.consumer-check-style-source",
    },
  ],
  [
    "consumerBuildStyleSource",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.consumerBuildStyleSource,
      inputContract: "ConsumerStyleSourceBuildInputV0",
      outputProduct: "omena-query.consumer-build-style-source",
    },
  ],
  [
    "consumerBuildStyleSources",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.consumerBuildStyleSources,
      inputContract: "ConsumerStyleSourcesBuildInputV0",
      outputProduct: "omena-query.consumer-build-style-source",
    },
  ],
  [
    "consumerTransformPassList",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.consumerTransformPassList,
      inputContract: "None",
      outputProduct: "omena-query.transform-pass-list",
    },
  ],
] as const);

void (async () => {
  const summary = await runShadowOmenaQuerySelectedQueryAdapterCapabilities();

  assert.equal(summary.schemaVersion, "0");
  assert.equal(summary.product, "omena-query.selected-query-adapter-capabilities");
  assert.equal(summary.defaultCandidateBackend, "rust-selected-query");
  assert.equal(summary.routingStatus, "runtimeBacked");
  assert.deepEqual(summary.schemaVersionPolicy, {
    schemaVersion: "0",
    product: "omena-query.schema-version-policy",
    currentVersion: "0",
    currentVersionLabel: "V0",
    acceptedVersions: ["0"],
    deprecatedVersions: [],
    rejectedVersionPolicy: "rejectUnknownVersionsBeforeExecution",
    missingVersionPolicy: "rejectMissingSchemaVersionOnExternalInputs",
    migrationPolicy: [
      "new versions require additive reader before writer",
      "old and new versions must run through the same omena-query facade during migration",
      "schema gate must include current accepted, missing, label-only, and future-version checks",
      "breaking payload changes require a new numeric schemaVersion and explicit migration adapter",
    ],
    compatibilityGate: "rust/omena-query/adapter-capabilities",
  });
  assert.deepEqual(
    summary.schemaVersionChecks.map((check) => ({
      requestedVersion: check.requestedVersion,
      accepted: check.accepted,
      status: check.status,
      migrationAction: check.migrationAction,
    })),
    [
      {
        requestedVersion: "0",
        accepted: true,
        status: "current",
        migrationAction: "executeCurrentFacade",
      },
      {
        requestedVersion: "V0",
        accepted: false,
        status: "labelOnlyVersionRejected",
        migrationAction: "sendNumericSchemaVersion",
      },
      {
        requestedVersion: "1",
        accepted: false,
        status: "unsupportedVersion",
        migrationAction: "rejectBeforeExecution",
      },
      {
        requestedVersion: null,
        accepted: false,
        status: "missingVersion",
        migrationAction: "rejectBeforeExecution",
      },
    ],
  );
  assert.deepEqual([...summary.requiredInputContracts].toSorted(), [
    "CompletionAtInputV0",
    "ConsumerStyleSourceBuildInputV0",
    "ConsumerStyleSourceInputV0",
    "ConsumerStyleSourcesBuildInputV0",
    "EngineInputV2",
    "OmenaParserStyleFactsInputV0",
    "ReadCascadeAtPositionInputV0",
    "ReadStyleContextIndexInputV0",
    "RefsForClassInputV0",
    "RenamePlanInputV0",
    "SourceDiagnosticsForFileInputV0",
    "StyleCodeActionsInputV0",
    "StyleDiagnosticsForFileInputV0",
    "StyleSemanticGraphBatchInputV0",
    "StyleSemanticGraphInputV0",
    "TransformContextFromEngineInputV0",
    "TransformContextInputV0",
    "TransformExecuteInputV0",
    "TransformPlanInputV0",
  ]);
  assert.deepEqual([...summary.expressionSemanticsPayloadContracts].toSorted(), [
    "valueDomainDerivation",
    "valueDomainKind",
    "valueDomainProvenanceTree",
  ]);
  assert.deepEqual([...summary.adapterReadiness].toSorted(), [
    "backendCapabilityMatrix",
    "canonicalProducerWrapperBoundary",
    "completionAtRunner",
    "consumerBuildFacade",
    "consumerCheckFacade",
    "consumerTransformPassListFacade",
    "expressionDomainCallSiteFlowAnalysisRunner",
    "expressionDomainControlFlowAnalysisRunner",
    "expressionDomainFlowAnalysisRunner",
    "expressionDomainProvenanceExplanationRunner",
    "expressionDomainSalsaRuntime",
    "expressionDomainSelectorProjection",
    "expressionSemanticsDerivationPayload",
    "fragmentBundleBoundary",
    "omenaParserStyleFactExtraction",
    "queryEvaluationRuntime",
    "readCascadeAtPosition",
    "readCascadeCustomPropertyLeastFixedPoint",
    "readStyleContextIndexRunner",
    "refsForClassRunner",
    "renamePlanRunner",
    "runnerCommandContract",
    "scssEvaluatorControlFlowFacade",
    "scssEvaluatorControlFlowOracleCorpusFacade",
    "semanticReachabilityTransformContext",
    "sourceDiagnosticsForFileRunner",
    "sourceResolutionRuntimeIndex",
    "staticStylesheetEvaluatorFacade",
    "staticStylesheetEvaluatorOracleCorpusFacade",
    "styleCodeActionsRunner",
    "styleDiagnosticsForFileRunner",
    "styleSemanticGraphBridgeBoundary",
    "transformContextProducer",
    "transformEggExecutionWitnesses",
    "transformExecutionRunner",
    "transformPlanRunner",
  ]);

  for (const backendKind of BACKEND_KINDS) {
    const declared = summary.backendKinds.find((backend) => backend.backendKind === backendKind);
    assert.ok(declared, `missing declared backend capability: ${backendKind}`);
    assert.deepEqual(
      {
        sourceResolution: declared.sourceResolution,
        expressionSemantics: declared.expressionSemantics,
        selectorUsage: declared.selectorUsage,
        styleSemanticGraph: declared.styleSemanticGraph,
      },
      {
        sourceResolution: usesRustSourceResolutionBackend(backendKind),
        expressionSemantics: usesRustExpressionSemanticsBackend(backendKind),
        selectorUsage: usesRustSelectorUsageBackend(backendKind),
        styleSemanticGraph: usesRustStyleSemanticGraphBackend(backendKind),
      },
      `backend capability drift: ${backendKind}`,
    );
  }

  assert.deepEqual(
    summary.backendKinds.map((backend) => backend.backendKind).toSorted(),
    [...BACKEND_KINDS].toSorted(),
  );

  for (const [surface, expected] of EXPECTED_RUNNER_COMMANDS) {
    const declared = summary.runnerCommands.find((command) => command.surface === surface);
    assert.ok(declared, `missing runner command declaration: ${surface}`);
    assert.deepEqual(
      {
        command: declared.command,
        inputContract: declared.inputContract,
        outputProduct: declared.outputProduct,
      },
      expected,
      `runner command drift: ${surface}`,
    );
  }

  assert.deepEqual(
    summary.runnerCommands.map((command) => command.surface).toSorted(),
    [...EXPECTED_RUNNER_COMMANDS.keys()].toSorted(),
  );

  const staticStylesheetOracleCorpus = runStaticStylesheetEvaluatorOracleCorpus();
  const staticStylesheetEvaluator = runStaticStylesheetEvaluator();
  assertStaticStylesheetEvaluator(staticStylesheetEvaluator);
  assertNativeEvaluatorCutoverProductPathPolicy();
  assertStaticStylesheetEvaluatorOracleCorpus(staticStylesheetOracleCorpus);
  const lessStaticLifExports = runStaticLifExports();
  assertLessStaticLifExports(lessStaticLifExports);
  const lessCrossFileSummary = runLessWorkspaceCrossFileSummary();
  assertLessWorkspaceCrossFileSummary(lessCrossFileSummary);
  assertLessUnifiedHypergraphEdgeKindCoverage();
  const scssControlFlow = runScssEvaluatorControlFlow();
  assertScssEvaluatorControlFlow(scssControlFlow);
  const scssControlFlowOracleCorpus = runScssEvaluatorControlFlowOracleCorpus();
  assertScssEvaluatorControlFlowOracleCorpus(scssControlFlowOracleCorpus);

  process.stdout.write(
    [
      "validated omena-query selected-query adapter capabilities:",
      `backends=${summary.backendKinds.length}`,
      `runnerCommands=${summary.runnerCommands.length}`,
      `staticStylesheetEvaluator=${staticStylesheetEvaluator.productOutputSource}`,
      `staticStylesheetOracleFixtures=${staticStylesheetOracleCorpus.fixtureCount}`,
      `lessLifExports=${lessStaticLifExports.lessSpecificExportCount}`,
      `lessCrossFileEdges=${lessCrossFileSummary.summaryEdgeCount}`,
      `scssControlFlowBlocks=${scssControlFlow.controlFlowBlockCount}`,
      `scssControlFlowOracleFixtures=${scssControlFlowOracleCorpus.fixtureCount}`,
      `routing=${summary.routingStatus}`,
    ].join(" "),
  );
  process.stdout.write("\n");
})();

function assertNativeEvaluatorCutoverProductPathPolicy(): void {
  const transformModelSource = readText("rust/crates/omena-transform-passes/src/model.rs");
  const queryEvaluationSource = readText(
    "rust/crates/omena-query/src/style/transform/static_stylesheet/evaluation_source.rs",
  );
  const transformExecutorSource = readText(
    "rust/crates/omena-transform-passes/src/runtime/executor.rs",
  );

  assertSourceIncludesAll(
    transformModelSource,
    [
      "self.oracle.as_ref().is_some_and(|oracle|",
      'oracle.mode == "oracleOnly"',
      "oracle.divergence_count == 0",
      "oracle.all_legacy_declaration_values_preserved",
      "self.oracle",
      ".is_some_and(|_| native_output == self.evaluated_css)",
    ],
    "transform module evaluation must require a retained oracle before native product output",
  );
  assertSourceExcludesAll(
    transformModelSource,
    ["oracle.as_ref().is_none_or", ".is_none_or(|_| native_output == self.evaluated_css)"],
    "transform module evaluation must not treat a missing oracle as product-output-ready",
  );

  assertSourceIncludesAll(
    queryEvaluationSource,
    [
      "if !evaluation.may_consume_native_product_output()",
      "evaluation.native_output_matches_retained_oracle(native_edit_output.as_str())",
      "materialize_transform_module_evaluation_native_edits(input_css, &evaluation.native_edits)",
      "native_css == evaluation.evaluated_css",
    ],
    "omena-query static stylesheet product output must stay behind the native evaluator cutover policy",
  );
  assertSourceExcludesAll(
    queryEvaluationSource,
    [
      "Some(evaluation.evaluated_css",
      "return Some(evaluation.evaluated_css",
      "Some(evaluation.evaluated_css.clone())",
    ],
    "omena-query static stylesheet product output must not blind-consume evaluated_css",
  );

  assertSourceIncludesAll(
    transformExecutorSource,
    [
      "if !evaluation.may_consume_native_product_output()",
      "evaluation.native_output_matches_retained_oracle(native_edit_output)",
      "apply_transform_module_evaluation_native_edits(input_css, &evaluation.native_edits)",
      "native_css == evaluation.evaluated_css",
    ],
    "transform executor product output must stay behind the native evaluator cutover policy",
  );
  assertSourceExcludesAll(
    transformExecutorSource,
    [
      "css: evaluation.evaluated_css",
      "css: evaluation.evaluated_css.clone()",
      "output_css: evaluation.evaluated_css",
      "output_css: evaluation.evaluated_css.clone()",
    ],
    "transform executor product output must not blind-consume evaluated_css",
  );
}

function readText(path: string): string {
  return readFileSync(path, "utf8");
}

function assertSourceIncludesAll(
  source: string,
  needles: readonly string[],
  message: string,
): void {
  for (const needle of needles) {
    assert.ok(source.includes(needle), `${message}: missing ${needle}`);
  }
}

function assertSourceExcludesAll(
  source: string,
  needles: readonly string[],
  message: string,
): void {
  for (const needle of needles) {
    assert.ok(!source.includes(needle), `${message}: forbidden ${needle}`);
  }
}

function runStaticStylesheetEvaluatorOracleCorpus(): StaticStylesheetEvaluatorOracleCorpusSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      SELECTED_QUERY_RUNNER_COMMANDS.staticStylesheetEvaluatorOracleCorpus,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `static stylesheet evaluator oracle corpus command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as StaticStylesheetEvaluatorOracleCorpusSummaryV0;
}

function runStaticStylesheetEvaluator(): StaticStylesheetEvaluatorSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      SELECTED_QUERY_RUNNER_COMMANDS.staticStylesheetEvaluator,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify({
        targetStylePath: "/tmp/Button.module.less",
        engineInput: {
          version: "static-stylesheet-evaluator-engine-input-v0",
          sources: [],
          styles: [
            {
              filePath: "/tmp/Button.module.less",
              source: "@brand: red;\n.button { color: @brand; }\n",
              document: {
                selectors: [],
              },
            },
          ],
          typeFacts: [],
        },
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `static stylesheet evaluator command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as StaticStylesheetEvaluatorSummaryV0;
}

function runStaticLifExports(): StaticLifExportsSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      SELECTED_QUERY_RUNNER_COMMANDS.staticLifExports,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify({
        targetStylePath: "/tmp/Button.module.less",
        engineInput: {
          version: "static-lif-engine-input-v0",
          sources: [],
          styles: [
            {
              filePath: "/tmp/Button.module.less",
              source:
                "@brand: #fff;\n@tokens: { primary: @brand; @gap: 2px; };\n.button(@gap: 1rem, @rest...) when (@gap > 0) { color: @brand; }\n",
              document: {
                selectors: [],
              },
            },
          ],
          typeFacts: [],
        },
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `static LIF exports command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as StaticLifExportsSummaryV0;
}

function runLessWorkspaceCrossFileSummary(): CrossFileSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "workspace-cross-file-summary",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify({
        styles: [
          {
            stylePath: "/tmp/tokens.less",
            styleSource: "@brand: red;",
          },
          {
            stylePath: "/tmp/Button.module.less",
            styleSource: '@import "./tokens.less"; .button { color: @brand; }',
          },
        ],
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `Less workspace cross-file summary command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as CrossFileSummaryV0;
}

function runScssEvaluatorControlFlowOracleCorpus(): ScssEvaluatorControlFlowOracleCorpusSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      SELECTED_QUERY_RUNNER_COMMANDS.scssEvaluatorControlFlowOracleCorpus,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `SCSS control-flow oracle corpus command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as ScssEvaluatorControlFlowOracleCorpusSummaryV0;
}

function runScssEvaluatorControlFlow(): ScssEvaluatorControlFlowSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      SELECTED_QUERY_RUNNER_COMMANDS.scssEvaluatorControlFlow,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify({
        targetStylePath: "/tmp/Button.module.scss",
        engineInput: {
          version: "scss-control-flow-engine-input-v0",
          sources: [],
          styles: [
            {
              filePath: "/tmp/Button.module.scss",
              source:
                "$brand: red;\n@mixin tone($enabled) { @if $enabled { color: $brand; } @else { color: blue; } }\n.button { @include tone(true); }\n",
              document: {
                selectors: [],
              },
            },
          ],
          typeFacts: [],
        },
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  assert.equal(
    result.status,
    0,
    `SCSS control-flow command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as ScssEvaluatorControlFlowSummaryV0;
}

function assertStaticStylesheetEvaluator(summary: StaticStylesheetEvaluatorSummaryV0): void {
  assert.equal(summary.product, "omena-query.static-stylesheet-evaluator");
  assert.equal(summary.mode, "oracleOnly");
  assert.equal(summary.dialect, "less");
  assert.equal(summary.valueType, "AbstractCssValueV0");
  assert.equal(summary.supportedDialect, true);
  assert.equal(summary.productOutputSource, "nativeEditOutput");
  assert.equal(summary.legacyOutputRetainedAsOracle, true);
  assert.equal(summary.legacyOutputConsumedUntilCutover, false);
  assert.equal(summary.evaluationAvailable, true);
  assert.equal(summary.valueResolutionAvailable, true);
  assert.equal(summary.divergenceCount, 0);
  assert.equal(summary.allLegacyDeclarationValuesPreserved, true);
  assert.equal(summary.nativeEditOutputMatchesEvaluatedCss, true);
  assert.ok(summary.nativeValueReferenceCount > 0);
  assert.ok(summary.nativeResolvedValueCount > 0);
  assert.ok(
    summary.nativeEditOutput?.includes("color: red"),
    "EngineInput-backed static evaluator must expose the native edit output",
  );
}

function assertScssEvaluatorControlFlow(summary: ScssEvaluatorControlFlowSummaryV0): void {
  assert.equal(summary.product, "omena-query.scss-evaluator-control-flow");
  assert.equal(summary.mode, "oracleOnly");
  assert.equal(summary.dialect, "scss");
  assert.equal(summary.valueType, "AbstractCssValueV0");
  assert.equal(summary.supportedDialect, true);
  assert.equal(summary.flatCssCfgBuilt, true);
  assert.equal(summary.mergedCrossFileGraph, false);
  assert.ok(summary.controlFlowBlockCount > 0);
  assert.ok(summary.controlFlowBranchBlockCount > 0);
  assert.ok(summary.callReturnNodeCount > 0);
  assert.equal(summary.valueAnalysisConverged, true);
  assert.equal(summary.pruneReachabilityAvailable, true);
  assert.equal(summary.pruneReachabilityConverged, true);
  assert.equal(summary.pruneReachabilityFlatCssCfgBuilt, true);
  assert.deepEqual(summary.readySurfaces.toSorted(), [
    "scssEvaluatorCallReturnIr",
    "scssEvaluatorControlFlowIr",
    "scssEvaluatorControlFlowPruneReachability",
    "scssEvaluatorControlFlowValueAnalysis",
  ]);
}

function assertLessStaticLifExports(summary: StaticLifExportsSummaryV0): void {
  assert.equal(summary.product, "omena-query.static-lif-exports");
  assert.equal(summary.mode, "staticInterfaceOnly");
  assert.equal(summary.dialect, "less");
  assert.equal(summary.sourceSyntax, "less");
  assert.equal(summary.sifSuperset, true);
  assert.equal(summary.lessSpecificExportCount, 3);
  assert.equal(summary.lessVariableCount, 1);
  assert.equal(summary.lessMixinCount, 1);
  assert.equal(summary.lessDetachedRulesetCount, 1);
  assert.deepEqual(summary.lessVariableNames, ["@brand"]);
  assert.deepEqual(summary.lessMixinNames, [".button"]);
  assert.deepEqual(summary.lessDetachedRulesetNames, ["@tokens"]);
  assert.equal(summary.sifVariableCount, 0);
  assert.equal(summary.exports.lessVariables[0]?.valueRepr, "#fff");
  assert.equal(summary.exports.lessMixins[0]?.guarded, true);
  assert.deepEqual(summary.exports.lessDetachedRulesets[0]?.memberNames.toSorted(), [
    "@gap",
    "primary",
  ]);
}

function assertLessWorkspaceCrossFileSummary(summary: CrossFileSummaryV0): void {
  assert.equal(summary.product, "omena-query.cross-file-summary");
  assert.equal(summary.status, "workspaceSummaryEdgeSeed");
  assert.equal(summary.summaryScope, "workspaceStyleAndSource");
  assert.equal(summary.styleCount, 2);
  assert.equal(summary.summaryEdgeCount, 2);
  assert.equal(summary.capabilities.sassModuleEdgesReady, true);
  assert.equal(summary.capabilities.stableSummaryHashReady, true);
  assert.equal(summary.capabilities.linearProvenanceReady, true);
  assert.equal(summary.capabilities.linearProvenanceRoundTripReady, true);

  const edgeKindCounts = new Map(
    summary.edgeKindCounts.map((entry) => [entry.edgeKind, entry.count]),
  );
  assert.equal(edgeKindCounts.get("lessImport"), 1);
  assert.equal(edgeKindCounts.get("lessModuleGraphClosure"), 1);
  assert.equal(edgeKindCounts.get("sassImport"), undefined);

  assert.ok(
    summary.edges.some(
      (edge) =>
        edge.edgeKind === "lessImport" &&
        edge.fromKind === "style" &&
        edge.fromPath === "/tmp/Button.module.less" &&
        edge.targetKind === "style" &&
        edge.targetPath === "/tmp/tokens.less" &&
        edge.source === "./tokens.less" &&
        edge.status === "resolved" &&
        edge.provenance.includes("omena-query.sass-module-cross-file-resolution") &&
        edge.provenance.includes("omena-parser.sass-module-facts"),
    ),
    "Less import must reach the workspace cross-file summary as a resolved Less edge",
  );
  assert.ok(
    summary.edges.some(
      (edge) =>
        edge.edgeKind === "lessModuleGraphClosure" &&
        edge.fromKind === "style" &&
        edge.fromPath === "/tmp/Button.module.less" &&
        edge.targetKind === "style" &&
        edge.targetPath === "/tmp/tokens.less" &&
        edge.status === "reachable" &&
        edge.provenance.includes("omena-query.sass-module-cross-file-resolution") &&
        edge.provenance.includes("omena-parser.sass-module-facts"),
    ),
    "Less module closure must reach the workspace cross-file summary as a reachable Less edge",
  );
}

function assertLessUnifiedHypergraphEdgeKindCoverage(): void {
  const crossFileSummarySource = readText("rust/crates/omena-cross-file-summary/src/lib.rs");
  assertSourceIncludesAll(
    crossFileSummarySource,
    [
      "LessImport",
      "LessModuleGraphClosure",
      'Self::LessImport => "lessImport"',
      'Self::LessModuleGraphClosure => "lessModuleGraphClosure"',
      '"lessImport" => UnifiedHypergraphEdgeKindV0::LessImport',
      '"lessModuleGraphClosure" => UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure',
    ],
    "Less module edges must stay consumable by the unified hypergraph substrate",
  );
}

function assertStaticStylesheetEvaluatorOracleCorpus(
  summary: StaticStylesheetEvaluatorOracleCorpusSummaryV0,
): void {
  assert.equal(summary.product, "omena-query.static-stylesheet-evaluator-oracle-corpus");
  assert.equal(summary.mode, "oracleOnly");
  assert.equal(summary.valueType, "AbstractCssValueV0");
  assert.equal(summary.productOutputSource, "nativeEditOutput");
  assert.ok(summary.fixtureCount >= 118, "static stylesheet oracle corpus must not shrink");
  assert.ok(summary.scssFixtureCount >= 38, "SCSS oracle fixture coverage must not shrink");
  assert.ok(summary.sassFixtureCount >= 32, "Sass oracle fixture coverage must not shrink");
  assert.ok(summary.lessFixtureCount >= 48, "Less oracle fixture coverage must not shrink");
  assert.equal(summary.evaluatedFixtureCount, summary.fixtureCount);
  assert.equal(summary.legacyOutputRetainedAsOracleCount, summary.evaluatedFixtureCount);
  assert.equal(summary.legacyOutputConsumedUntilCutoverCount, 0);
  assert.equal(summary.allLegacyOutputsRetainedAsOracle, true);
  assert.equal(summary.missingEvaluationCount, 0);
  assert.equal(summary.divergenceCount, 0);
  assert.equal(summary.nativeEditOutputMatchCount, summary.fixtureCount);
  assert.ok(summary.nativeValueEditCount > 0);
  assert.ok(summary.nativeStructuralEditCount > 0);
  assert.ok(summary.scssControlFlowValueTruthinessCount > 0);
  assert.equal(summary.scssControlFlowContextualTruthinessFallbackCount, 0);
  assert.equal(summary.scssControlFlowContextualTruthinessConflictCount, 0);
  assert.equal(
    summary.scssControlFlowPruneReachabilityFixtureCount,
    summary.scssFixtureCount + summary.sassFixtureCount,
  );
  assert.equal(
    summary.scssControlFlowPruneReachabilityFlatCssCfgBuiltCount,
    summary.scssFixtureCount + summary.sassFixtureCount,
  );
  assert.ok(summary.scssControlFlowPruneReachabilityChangedFixtureCount > 0);
  assert.ok(summary.nativeRawValueCount > 0);
  assert.ok(summary.nativeTopValueCount > 0);
  assert.ok(summary.nativeCycleValueCount > 0);
  assert.ok(summary.nativeFuelExhaustedValueCount > 0);
  assert.ok(summary.nativeUnresolvedReferenceValueCount > 0);
  assert.ok(summary.nativeUnsupportedDynamicValueCount > 0);
  assert.equal(summary.allLegacyDeclarationValuesPreserved, true);
  assert.equal(summary.allNativeEditOutputsMatchEvaluatedCss, true);
  assert.equal(summary.nativeProductOutputCorpusReady, true);

  const corpus = summary.corpus;
  assert.ok(corpus, "selected-query facade must expose the underlying evaluator corpus");
  assert.equal(corpus.product, "omena-scss-eval.static-stylesheet-oracle-corpus");
  assert.equal(corpus.fixtureCount, summary.fixtureCount);
  assert.equal(corpus.divergenceCount, 0);
  assert.equal(corpus.nativeEditOutputMatchCount, corpus.fixtureCount);
  assert.ok(corpus.scssControlFlowValueTruthinessCount > 0);
  assert.equal(corpus.scssControlFlowContextualTruthinessFallbackCount, 0);
  assert.equal(corpus.scssControlFlowContextualTruthinessConflictCount, 0);
  assert.equal(
    corpus.scssControlFlowPruneReachabilityFixtureCount,
    corpus.scssFixtureCount + corpus.sassFixtureCount,
  );
  assert.equal(
    corpus.scssControlFlowPruneReachabilityFlatCssCfgBuiltCount,
    corpus.scssFixtureCount + corpus.sassFixtureCount,
  );
  assert.ok(corpus.scssControlFlowPruneReachabilityChangedFixtureCount > 0);
  assert.ok(
    corpus.fixtures.some(
      (fixture) =>
        fixture.scssControlFlowPruneReachabilityHaveTerminalsChanged &&
        fixture.nativeEditOutputMatchesEvaluatedCss &&
        fixture.divergenceCount === 0,
    ),
    "static stylesheet product corpus must include an oracle-clean fixture with changed SCCP reachability",
  );
  assert.equal(corpus.legacyOutputConsumedUntilCutoverCount, 0);
  assert.equal(corpus.allLegacyDeclarationValuesPreserved, true);
  assert.equal(corpus.allNativeEditOutputsMatchEvaluatedCss, true);
  assert.equal(corpus.nativeProductOutputCorpusReady, true);
  assert.ok(
    corpus.fixtures.every(
      (fixture) =>
        fixture.productOutputSource === "nativeEditOutput" &&
        fixture.legacyOutputRetainedAsOracle &&
        !fixture.legacyOutputConsumedUntilCutover,
    ),
    "native evaluator corpus must retain legacy output only as oracle evidence",
  );

  const fixtures = new Map(corpus.fixtures.map((fixture) => [fixture.id, fixture]));
  for (const id of [
    "scss.dynamic-function-return",
    "scss.unresolved-forward-composite",
    "scss.recursive-function-return",
    "sass.variable-basic",
    "sass.static-function-return",
    "sass.static-mixin-include",
    "sass.static-if-return",
    "sass.static-for-return",
    "sass.static-while-return",
    "sass.static-each-return",
    "sass.static-each-tuple-function-source-return",
    "sass.static-map-list-builtins",
    "sass.static-default-function-arguments",
    "sass.static-default-argument-prior-parameter",
    "sass.static-named-default-argument-prior-parameter",
    "sass.static-named-function-arguments",
    "sass.static-named-argument-default-tail",
    "sass.static-hyphen-underscore-function-reference",
    "sass.static-hyphen-underscore-named-argument",
    "sass.static-named-mixin-arguments",
    "sass.static-named-mixin-default-tail",
    "sass.static-mixin-default-argument-prior-parameter",
    "sass.static-named-mixin-default-argument-prior-parameter",
    "sass.static-mixin-content-block",
    "sass.static-mixin-content-arguments",
    "sass.static-mixin-content-expression-arguments",
    "sass.static-mixin-content-nested-include",
    "sass.static-nested-mixin-include",
    "sass.static-hyphen-underscore-mixin-include",
    "scss.static-map-list-builtins",
    "scss.static-default-function-arguments",
    "scss.static-default-argument-prior-parameter",
    "scss.static-named-default-argument-prior-parameter",
    "scss.static-named-function-arguments",
    "scss.static-named-argument-default-tail",
    "scss.static-hyphen-underscore-function-reference",
    "scss.static-hyphen-underscore-named-argument",
    "scss.static-named-mixin-arguments",
    "scss.static-named-mixin-default-tail",
    "scss.static-mixin-default-argument-prior-parameter",
    "scss.static-named-mixin-default-argument-prior-parameter",
    "scss.static-mixin-content-block",
    "scss.static-mixin-content-arguments",
    "scss.static-mixin-content-expression-arguments",
    "scss.static-nested-mixin-include",
    "scss.static-hyphen-underscore-mixin-include",
    "scss.dynamic-top-level-if",
    "scss.indirect-recursive-function-return",
    "less.variable-basic",
    "less.dynamic-escaped-string",
    "less.quoted-value-interpolation",
    "less.escaped-quoted-value-interpolation",
    "less.fuel-exhausted-variable-chain",
    "less.static-mixin",
    "less.recursive-nested-mixin-call",
    "less.named-mixin-arguments",
    "less.semicolon-named-mixin-arguments",
    "less.variadic-mixin-arguments",
    "less.mixin-accessor",
    "less.unknown-mixin-accessor-member",
    "less.unknown-mixin-accessor-property-member",
    "less.namespace-mixin",
    "less.parameterized-namespace-mixin",
    "less.unbound-parameterized-namespace-mixin",
    "less.literal-pattern-mixin",
    "less.unmatched-literal-pattern-mixin",
    "less.important-mixin",
    "less.unknown-mixin-call-suffix",
    "less.default-guarded-mixin",
    "less.false-guarded-mixin",
    "less.false-type-predicate-guarded-mixin",
    "less.false-guarded-namespace-mixin",
    "less.guarded-namespace-mixin",
    "less.detached-ruleset",
    "less.unknown-detached-ruleset-mixin-call",
    "less.ruleset-guarded-mixin",
    "less.detached-ruleset-accessor",
    "less.unknown-detached-ruleset-accessor-member",
    "less.hsl-color-transforms",
    "less.relative-color-transforms",
    "less.convert-units",
    "less.trig-functions",
    "less.extended-numeric-builtins",
    "less.percentage-rounding-builtins",
    "less.range-list",
    "less.replace-string",
    "less.format-string",
    "less.isruleset-predicate",
    "less.isdefined-predicate",
    "less.property-isdefined-predicate",
    "less.property-variable-alias",
    "less.isdefined-guarded-mixin",
    "less.property-guarded-mixin",
    "less.future-property-guarded-mixin",
    "less.rgb-color-constructors",
    "less.color-mix",
    "less.color-blend",
    "less.hsv-color-metadata",
    "less.contrast-color",
  ]) {
    const fixture = fixtures.get(id);
    assert.ok(fixture, `missing oracle fixture ${id}`);
    assert.equal(fixture.evaluationAvailable, true, `oracle fixture ${id} must evaluate`);
    assert.equal(
      typeof fixture.nativeEditOutput,
      "string",
      `oracle fixture ${id} must expose native edit output bytes`,
    );
    assert.ok(
      fixture.nativeEditOutput.length > 0,
      `oracle fixture ${id} native edit output must not be empty`,
    );
    assert.equal(fixture.divergenceCount, 0, `oracle fixture ${id} must not diverge`);
    assert.equal(
      fixture.nativeEditOutputMatchesEvaluatedCss,
      true,
      `oracle fixture ${id} native edits must match legacy output`,
    );
  }

  const indeterminateFixtureIds = corpus.fixtures
    .filter(
      (fixture) =>
        fixture.nativeTopValueCount > 0 ||
        fixture.nativeRawValueCount > 0 ||
        fixture.nativeCycleValueCount > 0 ||
        fixture.nativeFuelExhaustedValueCount > 0 ||
        fixture.nativeUnresolvedReferenceValueCount > 0 ||
        fixture.nativeUnsupportedDynamicValueCount > 0,
    )
    .map((fixture) => fixture.id)
    .toSorted();
  assert.deepEqual(
    indeterminateFixtureIds,
    [
      "less.dynamic-escaped-string",
      "less.fuel-exhausted-variable-chain",
      "scss.dynamic-function-return",
      "scss.dynamic-top-level-if",
      "scss.indirect-recursive-function-return",
      "scss.recursive-function-return",
      "scss.unresolved-forward-composite",
    ],
    "Top/Raw native evaluator outputs must stay confined to explicit dynamic, cycle, fuel, or unresolved oracle fixtures",
  );

  const dynamicFunction = fixtures.get("scss.dynamic-function-return");
  assert.equal(dynamicFunction?.nativeTopValueCount, 1);
  assert.equal(dynamicFunction?.nativeUnsupportedDynamicValueCount, 1);
  const dynamicTopLevelIf = fixtures.get("scss.dynamic-top-level-if");
  assert.equal(dynamicTopLevelIf?.nativeRawValueCount, 1);
  assert.equal(dynamicTopLevelIf?.nativeUnsupportedDynamicValueCount, 1);
  const unresolvedComposite = fixtures.get("scss.unresolved-forward-composite");
  assert.equal(unresolvedComposite?.nativeTopValueCount, 1);
  assert.equal(unresolvedComposite?.nativeUnresolvedReferenceValueCount, 1);
  const recursiveFunction = fixtures.get("scss.recursive-function-return");
  assert.equal(recursiveFunction?.nativeTopValueCount, 1);
  assert.equal(recursiveFunction?.nativeCycleValueCount, 1);
  const indirectRecursiveFunction = fixtures.get("scss.indirect-recursive-function-return");
  assert.equal(indirectRecursiveFunction?.nativeTopValueCount, 1);
  assert.equal(indirectRecursiveFunction?.nativeCycleValueCount, 1);
  const fuelExhaustedLess = fixtures.get("less.fuel-exhausted-variable-chain");
  assert.equal(fuelExhaustedLess?.nativeTopValueCount, 1);
  assert.equal(fuelExhaustedLess?.nativeFuelExhaustedValueCount, 1);
  const dynamicLess = fixtures.get("less.dynamic-escaped-string");
  assert.equal(dynamicLess?.nativeRawValueCount, 1);
  assert.equal(dynamicLess?.nativeUnsupportedDynamicValueCount, 1);
  const staticScssIf = fixtures.get("scss.static-top-level-if");
  assert.equal(staticScssIf?.scssControlFlowPruneReachabilityAvailable, true);
  assert.equal(staticScssIf?.scssControlFlowPruneReachabilityConverged, true);
  assert.equal(staticScssIf?.scssControlFlowPruneReachabilityFlatCssCfgBuilt, true);
  assert.equal(staticScssIf?.scssControlFlowPruneReachabilityHaveTerminalsChanged, false);
  assert.equal(staticScssIf?.scssControlFlowPruneReachabilityUnreachableBlockCount, 0);
  const staticMixinIf = fixtures.get("scss.static-mixin-if");
  assert.ok((staticMixinIf?.scssControlFlowValueTruthinessCount ?? 0) > 0);
  assert.equal(staticMixinIf?.scssControlFlowContextualTruthinessFallbackCount, 0);
  assert.equal(staticMixinIf?.scssControlFlowContextualTruthinessConflictCount, 0);
  const dynamicScssIf = fixtures.get("scss.dynamic-top-level-if");
  assert.equal(dynamicScssIf?.scssControlFlowPruneReachabilityAvailable, true);
  assert.equal(dynamicScssIf?.scssControlFlowPruneReachabilityConverged, true);
  assert.equal(dynamicScssIf?.scssControlFlowPruneReachabilityFlatCssCfgBuilt, true);
  assert.equal(dynamicScssIf?.scssControlFlowPruneReachabilityHaveTerminalsChanged, false);
  assert.equal(dynamicScssIf?.scssControlFlowPruneReachabilityUnreachableBlockCount, 0);
}

function assertScssEvaluatorControlFlowOracleCorpus(
  summary: ScssEvaluatorControlFlowOracleCorpusSummaryV0,
): void {
  assert.equal(summary.product, "omena-query.scss-evaluator-control-flow-oracle-corpus");
  assert.equal(summary.mode, "oracleOnly");
  assert.equal(summary.valueType, "AbstractCssValueV0");
  assert.equal(summary.nodeKeyType, "StableNodeKeyV0");
  assert.ok(summary.recursionCap > 0, "SCSS call-return recursion cap must stay explicit");
  assert.ok(summary.fixtureCount >= 53, "SCSS control-flow oracle corpus must not shrink");
  assert.ok(summary.scssFixtureCount >= 26, "SCSS control-flow fixture coverage must not shrink");
  assert.ok(summary.sassFixtureCount >= 26, "Sass control-flow fixture coverage must not shrink");
  assert.ok(
    summary.supportedFixtureCount >= 52,
    "supported SCSS control-flow fixtures must not shrink",
  );
  assert.equal(summary.rejectedFlatCssFixtureCount, 1);
  assert.ok(summary.branchFixtureCount >= 5);
  assert.ok(summary.loopFixtureCount >= 6);
  assert.ok(summary.backEdgeFixtureCount >= 6);
  assert.ok(summary.callReturnFixtureCount >= 5);
  assert.ok(summary.resolvedCallReturnFixtureCount >= 4);
  assert.ok(summary.topCallReturnFixtureCount >= 1);
  assert.ok(summary.recursiveCallFixtureCount >= 1);
  assert.equal(summary.convergedValueAnalysisFixtureCount, summary.supportedFixtureCount);
  assert.equal(summary.wideningWitness.product, "omena-scss-eval.control-flow-widening-witness");
  assert.equal(summary.wideningWitness.mode, "oracleOnly");
  assert.equal(summary.wideningWitness.valueType, "AbstractCssValueV0");
  assert.equal(summary.wideningWitness.policy, "nonConvergedOutputsWidenToTop");
  assert.equal(summary.wideningWitness.converged, false);
  assert.equal(summary.wideningWitnessConverged, false);
  assert.equal(summary.wideningWitness.iterationCount, summary.wideningWitness.maxIterations);
  assert.equal(summary.wideningWitnessWidenedToTopCount, summary.wideningWitness.nodeCount);
  assert.equal(summary.wideningWitness.outputTopCount, summary.wideningWitness.nodeCount);
  assert.equal(summary.pruneReachabilityFixtureCount, summary.supportedFixtureCount);
  assert.equal(summary.pruneReachabilityFlatCssCfgBuiltCount, summary.supportedFixtureCount);
  assert.ok(summary.pruneReachabilityChangedFixtureCount > 0);
  assert.equal(summary.flatCssCfgBuiltCount, summary.supportedFixtureCount);
  assert.equal(summary.mergedCrossFileGraphCount, 0);
  assert.equal(summary.allSupportedFixturesConverged, true);
  assert.equal(summary.noFlatCssCfgBuilt, false);
  assert.equal(summary.noMergedCrossFileGraph, true);

  const corpus = summary.corpus;
  assert.ok(corpus, "selected-query facade must expose the underlying control-flow corpus");
  assert.equal(corpus.product, "omena-scss-eval.control-flow-oracle-corpus");
  assert.equal(corpus.fixtureCount, summary.fixtureCount);
  assert.equal(corpus.noFlatCssCfgBuilt, false);
  assert.equal(corpus.noMergedCrossFileGraph, true);
  assert.equal(corpus.wideningWitness.widenedToTopCount, corpus.wideningWitness.nodeCount);

  const fixtures = new Map(corpus.fixtures.map((fixture) => [fixture.id, fixture]));
  assertControlFlowFixture(fixtures, "scss.branch-if-else", (fixture) => {
    assert.equal(fixture.supportedDialect, true);
    assert.ok(fixture.branchBlockCount > 0);
    assert.equal(fixture.valueAnalysisConverged, true);
  });
  assertControlFlowFixture(fixtures, "scss.static-while-loop", (fixture) => {
    assert.ok(fixture.loopBlockCount > 0);
    assert.ok(fixture.backEdgeCount > 0);
    assert.ok(fixture.valueAnalysisIterationCount > 0);
  });
  assertControlFlowFixture(fixtures, "scss.static-for-return", (fixture) => {
    assert.ok(fixture.callResolvedReturnValueCount > 0);
    assert.ok(fixture.exactCallResolvedReturnValueCount > 0);
  });
  for (const id of [
    "scss.static-default-function-arguments",
    "scss.static-default-argument-prior-parameter",
    "scss.static-named-default-argument-prior-parameter",
    "sass.static-default-function-arguments",
    "sass.static-default-argument-prior-parameter",
    "sass.static-named-default-argument-prior-parameter",
    "scss.static-named-function-arguments",
    "scss.static-named-argument-default-tail",
    "sass.static-named-function-arguments",
    "sass.static-named-argument-default-tail",
    "scss.static-hyphen-underscore-function-reference",
    "scss.static-hyphen-underscore-named-argument",
    "sass.static-hyphen-underscore-function-reference",
    "sass.static-hyphen-underscore-named-argument",
  ]) {
    assertControlFlowFixture(fixtures, id, (fixture) => {
      assert.equal(fixture.callResolvedReturnValueCount, 1);
      assert.equal(fixture.exactCallResolvedReturnValueCount, 1);
      assert.equal(fixture.valueAnalysisConverged, true);
    });
  }
  assertControlFlowFixture(fixtures, "scss.static-for-exclusive-bound", (fixture) => {
    assert.ok(fixture.callResolvedReturnValueCount > 0);
    assert.ok(fixture.exactCallResolvedReturnValueCount > 0);
    assert.equal(fixture.valueAnalysisConverged, true);
  });
  assertControlFlowFixture(fixtures, "sass.static-for-exclusive-bound", (fixture) => {
    assert.ok(fixture.callResolvedReturnValueCount > 0);
    assert.ok(fixture.exactCallResolvedReturnValueCount > 0);
    assert.equal(fixture.valueAnalysisConverged, true);
  });
  assertControlFlowFixture(fixtures, "scss.dynamic-loop-top", (fixture) => {
    assert.ok(fixture.topCallResolvedReturnValueCount > 0);
  });
  assertControlFlowFixture(fixtures, "scss.recursive-mixin-cap", (fixture) => {
    assert.ok(fixture.recursiveEdgeCount > 0);
    assert.ok(fixture.cappedRecursiveCallCount > 0);
  });
  for (const id of [
    "scss.static-named-mixin-arguments",
    "scss.static-named-mixin-default-tail",
    "scss.static-mixin-default-argument-prior-parameter",
    "scss.static-named-mixin-default-argument-prior-parameter",
    "scss.static-mixin-content-block",
    "scss.static-mixin-content-arguments",
    "scss.static-mixin-content-expression-arguments",
    "scss.static-hyphen-underscore-mixin-include",
    "sass.static-named-mixin-arguments",
    "sass.static-named-mixin-default-tail",
    "sass.static-mixin-default-argument-prior-parameter",
    "sass.static-named-mixin-default-argument-prior-parameter",
    "sass.static-mixin-content-block",
    "sass.static-mixin-content-arguments",
    "sass.static-mixin-content-expression-arguments",
    "sass.static-hyphen-underscore-mixin-include",
  ]) {
    assertControlFlowFixture(fixtures, id, (fixture) => {
      assert.equal(fixture.callReturnAvailable, true);
      assert.equal(fixture.callReturnEdgeCount, 1);
      assert.equal(fixture.valueAnalysisConverged, true);
    });
  }
  assertControlFlowFixture(fixtures, "scss.static-mixin-content-nested-include", (fixture) => {
    assert.equal(fixture.callReturnAvailable, true);
    assert.equal(fixture.callReturnEdgeCount, 2);
    assert.equal(fixture.valueAnalysisConverged, true);
  });
  assertControlFlowFixture(fixtures, "sass.static-mixin-content-nested-include", (fixture) => {
    assert.equal(fixture.callReturnAvailable, true);
    assert.equal(fixture.callReturnEdgeCount, 2);
    assert.equal(fixture.valueAnalysisConverged, true);
  });
  for (const id of ["scss.static-nested-mixin-include", "sass.static-nested-mixin-include"]) {
    assertControlFlowFixture(fixtures, id, (fixture) => {
      assert.equal(fixture.callReturnAvailable, true);
      assert.equal(fixture.callReturnEdgeCount, 2);
      assert.equal(fixture.valueAnalysisConverged, true);
    });
  }
  assertControlFlowFixture(fixtures, "css.flat-rejected", (fixture) => {
    assert.equal(fixture.supportedDialect, false);
    assert.equal(fixture.controlFlowAvailable, false);
    assert.equal(fixture.valueAnalysisAvailable, false);
    assert.equal(fixture.callReturnAvailable, false);
  });
}

function assertControlFlowFixture(
  fixtures: Map<string, ScssEvaluatorControlFlowOracleFixtureSummaryV0>,
  id: string,
  assertFixture: (fixture: ScssEvaluatorControlFlowOracleFixtureSummaryV0) => void,
): void {
  const fixture = fixtures.get(id);
  assert.ok(fixture, `missing SCSS control-flow oracle fixture ${id}`);
  assert.equal(
    fixture.flatCssCfgBuilt,
    fixture.supportedDialect,
    `fixture ${id} flat CSS CFG availability must track dialect support`,
  );
  assert.equal(
    fixture.mergedCrossFileGraph,
    false,
    `fixture ${id} must not merge cross-file and in-file graphs`,
  );
  assertFixture(fixture);
}
