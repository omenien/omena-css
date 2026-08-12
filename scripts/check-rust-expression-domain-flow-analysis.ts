import {
  runShadowExpressionDomainCallSiteFlowAnalysisInput,
  runShadowExpressionDomainControlFlowAnalysisInput,
  runShadowExpressionDomainFlowAnalysisInput,
  runShadowExpressionDomainProvenanceExplanationsInput,
  runShadowExpressionDomainReducedProductIterationInput,
  runShadowExpressionDomainSelectorProjectionInput,
  runShadowExpressionSemanticsCanonicalProducerInput,
  runShadowSelectorUsageCanonicalProducerInput,
  runShadowSourceResolutionCanonicalProducerInput,
  type EngineInputV2,
  type StringTypeFactsV2,
} from "./rust-shadow-shared";

const PRE_EXISTING_FLOW_HEDGE_LAYERS = [
  "FlowIterationLimit provenance",
  "converged/iterationCount fields",
  "checker-CLI converged=N/M counter",
  "human flowIterationWidening label and TypeScript mirror",
] as const;

const INPUT: EngineInputV2 = {
  version: "2",
  workspace: {
    root: "/tmp/cme-expression-domain-flow-analysis",
    classnameTransform: "asIs",
    settingsKey: "synthetic-expression-domain-flow-analysis",
  },
  sources: [],
  styles: [],
  typeFacts: [
    fact("expr-branch-a", {
      kind: "exact",
      values: ["btn-primary"],
    }),
    fact("expr-branch-b", {
      kind: "exact",
      values: ["btn-secondary"],
    }),
    fact("expr-branch-c", {
      kind: "exact",
      values: ["card"],
    }),
    factInFile("/tmp/Card.tsx", "expr-card-only", {
      kind: "exact",
      values: ["card-standalone"],
    }),
  ],
};

const REDUCED_PRODUCT_INPUT: EngineInputV2 = {
  version: "2",
  workspace: {
    root: "/tmp/cme-expression-domain-reduced-product",
    classnameTransform: "asIs",
    settingsKey: "synthetic-expression-domain-reduced-product",
  },
  sources: [],
  styles: [],
  typeFacts: [
    fact("expr-reduced", {
      kind: "constrained",
      constraintKind: "composite",
      prefix: "btn-",
      suffix: "-active",
      charMust: "a",
      charMay: "-abceintv",
      mayIncludeOtherChars: false,
    }),
  ],
};

const CERTAINTY_HEDGE_INPUT: EngineInputV2 = {
  version: "2",
  workspace: {
    root: "/tmp/cme-expression-domain-certainty-hedge",
    classnameTransform: "asIs",
    settingsKey: "synthetic-expression-domain-certainty-hedge",
  },
  sources: [
    {
      filePath: "/tmp/Certainty.tsx",
      document: {
        classExpressions: [
          {
            id: "expr-certainty-exact",
            kind: "styleAccess",
            scssModulePath: "/tmp/Certainty.module.scss",
            range: range(0, 0, 0, 11),
            className: "x",
            rootBindingDeclId: null,
            accessPath: ["styles", "x"],
          },
        ],
      },
      bindingGraph: {
        declarations: [],
        resolutions: [],
      },
    },
  ],
  styles: [
    {
      filePath: "/tmp/Certainty.module.scss",
      document: {
        selectors: [
          {
            name: "x",
            viewKind: "canonical",
            canonicalName: "x",
            range: range(0, 0, 0, 11),
            nestedSafety: "safe",
            composes: null,
            bemSuffix: null,
          },
        ],
      },
    },
  ],
  typeFacts: [
    {
      filePath: "/tmp/Certainty.tsx",
      expressionId: "expr-certainty-exact",
      facts: {
        kind: "exact",
        values: ["x"],
      },
      controlFlowGraph: {
        entryBlockId: "seed",
        blocks: [
          {
            id: "seed",
            kind: "assignment",
            transferKind: "assignFacts",
            successorBlockIds: ["loop"],
            boundaryEffect: "unknownBoundary",
            facts: {
              kind: "finiteSet",
              values: ["a", "b"],
            },
          },
          {
            id: "loop",
            kind: "loopBody",
            transferKind: "concatFacts",
            successorBlockIds: ["loop"],
            boundaryEffect: "unknownBoundary",
          },
        ],
      },
    },
  ],
};

void (async () => {
  process.stdout.write("== rust-expression-domain-flow-analysis:synthetic ==\n");

  const summary = await runShadowExpressionDomainFlowAnalysisInput(INPUT);
  const callSiteSummary = await runShadowExpressionDomainCallSiteFlowAnalysisInput(INPUT);
  const provenanceSummary = await runShadowExpressionDomainProvenanceExplanationsInput(INPUT);
  const reducedProductSummary =
    await runShadowExpressionDomainReducedProductIterationInput(REDUCED_PRODUCT_INPUT);
  const certaintyControlFlow =
    await runShadowExpressionDomainControlFlowAnalysisInput(CERTAINTY_HEDGE_INPUT);
  const certaintyProjection =
    await runShadowExpressionDomainSelectorProjectionInput(CERTAINTY_HEDGE_INPUT);
  const certaintyExpressionSemantics =
    await runShadowExpressionSemanticsCanonicalProducerInput(CERTAINTY_HEDGE_INPUT);
  const certaintySourceResolution =
    await runShadowSourceResolutionCanonicalProducerInput(CERTAINTY_HEDGE_INPUT);
  const certaintySelectorUsage =
    await runShadowSelectorUsageCanonicalProducerInput(CERTAINTY_HEDGE_INPUT);
  const branchAGraph = findAnalysis(summary, "/tmp/App.tsx:expr-branch-a:expression-domain-flow");
  const branchANode = findNode(branchAGraph, "expr-branch-a");
  const cardOnlyGraph = findAnalysis(
    summary,
    "/tmp/Card.tsx:expr-card-only:expression-domain-flow",
  );
  const cardOnlyNode = findNode(cardOnlyGraph, "expr-card-only");

  assertEqual(summary.product, "engine-input-producers.expression-domain-flow-analysis", "product");
  assertEqual(summary.analyses.length, INPUT.typeFacts.length, "per-expression graph count");
  assertEqual(
    summary.analyses.every((entry) => entry.analysis.contextSensitivity === "perSuppliedGraph"),
    true,
    "context sensitivity",
  );
  assertEqual(
    summary.analyses.every((entry) => entry.analysis.converged),
    true,
    "flow convergence",
  );
  assertEqual(
    summary.analyses.every((entry) =>
      entry.analysis.nodes.every((node) => node.id !== "file-merge"),
    ),
    true,
    "no synthetic file-merge node",
  );
  assertEqual(branchAGraph.filePath, "/tmp/App.tsx", "branch graph file path");
  assertEqual(branchAGraph.analysis.nodes.length, 1, "branch graph node count");
  assertEqual(branchANode.transferKind, "assignFacts", "branch transfer kind");
  assertEqual(branchANode.valueKind, "exact", "branch value kind");
  assertEqual(
    JSON.stringify(branchANode.value),
    JSON.stringify({
      kind: "exact",
      value: "btn-primary",
    }),
    "branch abstract value",
  );
  assertEqual(cardOnlyGraph.filePath, "/tmp/Card.tsx", "card graph file path");
  assertEqual(cardOnlyGraph.analysis.nodes.length, 1, "card graph node count");
  assertEqual(cardOnlyNode.transferKind, "assignFacts", "card transfer kind");
  assertEqual(cardOnlyNode.valueKind, "exact", "card value kind");
  assertEqual(
    JSON.stringify(cardOnlyNode.value),
    JSON.stringify({
      kind: "exact",
      value: "card-standalone",
    }),
    "card abstract value",
  );
  assertEqual(
    callSiteSummary.product,
    "engine-input-producers.expression-domain-call-site-flow-analysis",
    "call-site product",
  );
  assertEqual(callSiteSummary.zeroCfa.contextSensitivity, "0-cfa", "zero-cfa context");
  assertEqual(callSiteSummary.oneCfa.contextSensitivity, "1-cfa", "one-cfa context");
  assertEqual(callSiteSummary.zeroCfa.callSiteCount, 4, "multi-expression call-site count");
  assertEqual(callSiteSummary.oneCfa.callSiteCount, 4, "multi-expression 1-cfa call-site count");
  assertEqual(
    callSiteSummary.zeroCfa.entries.every(
      (entry) => entry.contextKey === "expression-domain-class-value@<root>",
    ),
    true,
    "zero-cfa root context keys",
  );
  assertEqual(
    callSiteSummary.oneCfa.entries.some(
      (entry) =>
        entry.contextKey ===
        "expression-domain-class-value@/tmp/App.tsx:expr-branch-a:expression-domain-flow",
    ),
    true,
    "one-cfa branch graph context key",
  );
  assertEqual(
    callSiteSummary.oneCfa.entries.some(
      (entry) =>
        entry.contextKey ===
        "expression-domain-class-value@/tmp/Card.tsx:expr-card-only:expression-domain-flow",
    ),
    true,
    "one-cfa card graph context key",
  );
  assertEqual(
    reducedProductSummary.product,
    "engine-input-producers.expression-domain-reduced-product-iteration",
    "reduced product iteration product",
  );
  assertEqual(reducedProductSummary.iterationCount, 1, "reduced product iteration count");
  assertEqual(
    reducedProductSummary.iterations[0]?.axisConstraintCount,
    3,
    "reduced product axis constraint count",
  );
  assertEqual(
    reducedProductSummary.iterations[0]?.iteration.converged,
    true,
    "reduced product iteration convergence",
  );
  assertEqual(
    reducedProductSummary.iterations[0]?.iteration.monotoneWitnessValid,
    true,
    "reduced product monotone witness",
  );
  assertEqual(
    reducedProductSummary.iterations[0]?.iteration.resultKind,
    "composite",
    "reduced product result kind",
  );
  assertEqual(
    provenanceSummary.product,
    "engine-input-producers.expression-domain-provenance-explanations",
    "provenance product",
  );
  assertEqual(provenanceSummary.explanationCount, 4, "provenance explanation count");
  assertEqual(
    provenanceSummary.explanations[0]?.derivation.product,
    "omena-abstract-value.reduced-class-value-derivation",
    "provenance derivation product",
  );
  assertEqual(
    provenanceSummary.explanations[0]?.provenanceTree.product,
    "omena-abstract-value.provenance-tree",
    "provenance tree product",
  );
  assertEqual(
    provenanceSummary.explanations[0]?.provenanceTree.root.operation,
    "exactLiteral",
    "provenance root operation",
  );

  const certaintyCases = [
    {
      expressionId: "expr-certainty-exact",
      selectorName: "x",
      baseCertainty: "exact",
      valueKind: "exact",
    },
  ] as const;
  const certaintyGraphIds: string[] = [];
  for (const certaintyCase of certaintyCases) {
    const certaintyGraphId = `/tmp/Certainty.tsx:${certaintyCase.expressionId}:expression-domain-control-flow`;
    certaintyGraphIds.push(certaintyGraphId);
    const certaintyControlEntry = certaintyControlFlow.analyses.find(
      (entry) => entry.graphId === certaintyGraphId,
    );
    if (!certaintyControlEntry) {
      throw new Error(`missing selector-certainty control-flow graph: ${certaintyGraphId}`);
    }
    const certaintyProjectionEntry = certaintyProjection.projections.find(
      (entry) =>
        entry.filePath === "/tmp/Certainty.tsx" && entry.nodeId === certaintyCase.expressionId,
    );
    if (!certaintyProjectionEntry) {
      throw new Error(`missing selector-certainty projection for graph: ${certaintyGraphId}`);
    }
    const expressionCandidate = certaintyExpressionSemantics.canonicalBundle.candidates.find(
      (entry) => entry.queryId === certaintyCase.expressionId,
    );
    if (!expressionCandidate) {
      throw new Error(
        `missing expression-semantics certainty product: ${certaintyCase.expressionId}`,
      );
    }
    const sourceCandidate = certaintySourceResolution.canonicalBundle.candidates.find(
      (entry) => entry.queryId === certaintyCase.expressionId,
    );
    if (!sourceCandidate) {
      throw new Error(`missing source-resolution certainty product: ${certaintyCase.expressionId}`);
    }
    const usageCandidate = certaintySelectorUsage.canonicalBundle.candidates.find(
      (entry) => entry.queryId === certaintyCase.selectorName,
    );
    if (!usageCandidate) {
      throw new Error(`missing selector-usage certainty product: ${certaintyCase.selectorName}`);
    }

    assertEqual(
      certaintyControlEntry.analysis.flowAnalysis.converged,
      false,
      `selector-certainty source-CFG convergence graph=${certaintyGraphId}`,
    );
    assertEqual(
      certaintyControlEntry.analysis.flowAnalysis.nodes.every(
        (node) => node.value.kind === "top" && node.value.provenance === "flowIterationLimit",
      ),
      true,
      `selector-certainty FlowIterationLimit provenance graph=${certaintyGraphId}`,
    );
    assertEqual(
      JSON.stringify({
        valueKind: certaintyProjectionEntry.valueKind,
        selectorNames: certaintyProjectionEntry.selectorNames,
        certainty: certaintyProjectionEntry.certainty,
      }),
      JSON.stringify({
        valueKind: certaintyCase.valueKind,
        selectorNames: [certaintyCase.selectorName],
        certainty: "possible",
      }),
      `selector-certainty typed query demotion ${certaintyCase.baseCertainty}->possible graph=${certaintyGraphId}`,
    );
    for (const [product, candidate] of [
      ["expression-semantics", expressionCandidate],
      ["source-resolution", sourceCandidate],
    ] as const) {
      assertEqual(
        JSON.stringify({
          selectorNames: candidate.selectorNames,
          certainty: candidate.selectorCertainty,
          shapeKind: candidate.selectorCertaintyShapeKind,
          shapeLabel: candidate.selectorCertaintyShapeLabel,
        }),
        JSON.stringify({
          selectorNames: [certaintyCase.selectorName],
          certainty: "possible",
          shapeKind: "unknown",
          shapeLabel: "unknown",
        }),
        `${product} certainty demotion ${certaintyCase.baseCertainty}->possible graph=${certaintyGraphId}`,
      );
    }
    assertEqual(
      JSON.stringify({
        totalReferences: usageCandidate.totalReferences,
        exactReferences: usageCandidate.exactReferenceCount,
        inferredOrBetterReferences: usageCandidate.inferredOrBetterReferenceCount,
      }),
      JSON.stringify({
        totalReferences: 1,
        exactReferences: 0,
        inferredOrBetterReferences: 0,
      }),
      `selector-usage certainty aggregate demotion ${certaintyCase.baseCertainty}->possible graph=${certaintyGraphId}`,
    );
  }
  const certaintyHedgedGraphCount = certaintyProjection.projections.filter(
    (entry) => entry.certainty === "possible",
  ).length;
  assertEqual(
    certaintyHedgedGraphCount >= certaintyCases.length,
    true,
    `selector-certainty product census floor graphs=${certaintyGraphIds.join("|")}`,
  );

  process.stdout.write(
    `validated expression-domain flow analysis: graphs=${summary.analyses.length} nodes=${summary.analyses.reduce((count, entry) => count + entry.analysis.nodes.length, 0)} callSiteProduct=${callSiteSummary.product} reducedProduct=${reducedProductSummary.product} provenance=${provenanceSummary.product} certaintyHedgedGraphs=${certaintyHedgedGraphCount} certaintyGraphIds=${certaintyGraphIds.join("|")} certaintyDemotion=exact->possible certaintyProducts=query|expression-semantics|source-resolution|selector-usage preExistingLayers=${PRE_EXISTING_FLOW_HEDGE_LAYERS.join("|")}\n`,
  );
})();

function findAnalysis(
  summary: Awaited<ReturnType<typeof runShadowExpressionDomainFlowAnalysisInput>>,
  graphId: string,
) {
  const entry = summary.analyses.find((candidate) => candidate.graphId === graphId);
  if (!entry) {
    throw new Error(`missing flow analysis graph: ${graphId}`);
  }
  return entry;
}

function findNode(
  entry: Awaited<ReturnType<typeof runShadowExpressionDomainFlowAnalysisInput>>["analyses"][number],
  nodeId: string,
) {
  const node = entry.analysis.nodes.find((candidate) => candidate.id === nodeId);
  if (!node) {
    throw new Error(`missing flow analysis node: ${nodeId}`);
  }
  return node;
}

function fact(expressionId: string, facts: StringTypeFactsV2) {
  return factInFile("/tmp/App.tsx", expressionId, facts);
}

function factInFile(filePath: string, expressionId: string, facts: StringTypeFactsV2) {
  return {
    filePath,
    expressionId,
    facts,
  };
}

function range(startLine: number, startCharacter: number, endLine: number, endCharacter: number) {
  return {
    start: { line: startLine, character: startCharacter },
    end: { line: endLine, character: endCharacter },
  };
}

function assertEqual(actual: unknown, expected: unknown, label: string): void {
  if (actual !== expected) {
    throw new Error(
      `${label}\nactual=${JSON.stringify(actual)}\nexpected=${JSON.stringify(expected)}`,
    );
  }
}
