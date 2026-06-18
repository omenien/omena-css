import { strict as assert } from "node:assert";
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
    "semanticReachabilityTransformContext",
    "sourceDiagnosticsForFileRunner",
    "sourceResolutionRuntimeIndex",
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

  process.stdout.write(
    [
      "validated omena-query selected-query adapter capabilities:",
      `backends=${summary.backendKinds.length}`,
      `runnerCommands=${summary.runnerCommands.length}`,
      `routing=${summary.routingStatus}`,
    ].join(" "),
  );
  process.stdout.write("\n");
})();
