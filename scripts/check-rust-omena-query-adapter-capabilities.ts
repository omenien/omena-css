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
      outputProduct: "omena-query.omena-parser-style-facts",
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
    "transformExecute",
    {
      command: SELECTED_QUERY_RUNNER_COMMANDS.transformExecute,
      inputContract: "TransformExecuteInputV0",
      outputProduct: "omena-query.transform-execute",
    },
  ],
] as const);

void (async () => {
  const summary = await runShadowOmenaQuerySelectedQueryAdapterCapabilities();

  assert.equal(summary.schemaVersion, "0");
  assert.equal(summary.product, "omena-query.selected-query-adapter-capabilities");
  assert.equal(summary.defaultCandidateBackend, "rust-selected-query");
  assert.equal(summary.routingStatus, "runtimeBacked");
  assert.deepEqual([...summary.requiredInputContracts].toSorted(), [
    "EngineInputV2",
    "OmenaParserStyleFactsInputV0",
    "ReadCascadeAtPositionInputV0",
    "StyleSemanticGraphBatchInputV0",
    "StyleSemanticGraphInputV0",
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
    "expressionDomainControlFlowAnalysisRunner",
    "expressionDomainFlowAnalysisRunner",
    "expressionDomainSalsaRuntime",
    "expressionDomainSelectorProjection",
    "expressionSemanticsDerivationPayload",
    "fragmentBundleBoundary",
    "omenaParserStyleFactExtraction",
    "readCascadeAtPosition",
    "runnerCommandContract",
    "sourceResolutionRuntimeIndex",
    "styleSemanticGraphBridgeBoundary",
    "transformContextProducer",
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
