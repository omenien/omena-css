import {
  runShadowExpressionDomainCallSiteFlowAnalysisInput,
  runShadowExpressionDomainFlowAnalysisInput,
  runShadowExpressionDomainProvenanceExplanationsInput,
  runShadowExpressionDomainReducedProductIterationInput,
  type EngineInputV2,
  type StringTypeFactsV2,
} from "./rust-shadow-shared";

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

void (async () => {
  process.stdout.write("== rust-expression-domain-flow-analysis:synthetic ==\n");

  const summary = await runShadowExpressionDomainFlowAnalysisInput(INPUT);
  const callSiteSummary = await runShadowExpressionDomainCallSiteFlowAnalysisInput(INPUT);
  const provenanceSummary = await runShadowExpressionDomainProvenanceExplanationsInput(INPUT);
  const reducedProductSummary =
    await runShadowExpressionDomainReducedProductIterationInput(REDUCED_PRODUCT_INPUT);
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
    summary.analyses.every((entry) => entry.analysis.contextSensitivity === "1-cfa"),
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

  process.stdout.write(
    `validated expression-domain flow analysis: graphs=${summary.analyses.length} nodes=${summary.analyses.reduce((count, entry) => count + entry.analysis.nodes.length, 0)} callSiteProduct=${callSiteSummary.product} reducedProduct=${reducedProductSummary.product} provenance=${provenanceSummary.product}\n`,
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

function assertEqual(actual: unknown, expected: unknown, label: string): void {
  if (actual !== expected) {
    throw new Error(
      `${label}\nactual=${JSON.stringify(actual)}\nexpected=${JSON.stringify(expected)}`,
    );
  }
}
