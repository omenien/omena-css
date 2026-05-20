import {
  runShadowExpressionDomainCallSiteFlowAnalysisInput,
  runShadowExpressionDomainFlowAnalysisInput,
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

void (async () => {
  process.stdout.write("== rust-expression-domain-flow-analysis:synthetic ==\n");

  const summary = await runShadowExpressionDomainFlowAnalysisInput(INPUT);
  const callSiteSummary = await runShadowExpressionDomainCallSiteFlowAnalysisInput(INPUT);
  const analysis = summary.analyses[0]?.analysis;
  const merge = analysis?.nodes.find((node) => node.id === "file-merge");

  assertEqual(summary.product, "engine-input-producers.expression-domain-flow-analysis", "product");
  assertEqual(analysis?.contextSensitivity, "1-cfa", "context sensitivity");
  assertEqual(analysis?.converged, true, "flow convergence");
  assertEqual(merge?.transferKind, "join", "merge transfer kind");
  assertEqual(merge?.valueKind, "finiteSet", "merge value kind");
  assertEqual(
    JSON.stringify(merge?.value),
    JSON.stringify({
      kind: "finiteSet",
      values: ["btn-primary", "btn-secondary", "card"],
    }),
    "merge abstract value",
  );
  assertEqual(
    callSiteSummary.product,
    "engine-input-producers.expression-domain-call-site-flow-analysis",
    "call-site product",
  );
  assertEqual(callSiteSummary.zeroCfa.contextSensitivity, "0-cfa", "zero-cfa context");
  assertEqual(callSiteSummary.oneCfa.contextSensitivity, "1-cfa", "one-cfa context");
  assertEqual(callSiteSummary.zeroCfa.callSiteCount, 2, "multi-file call-site count");
  assertEqual(callSiteSummary.oneCfa.callSiteCount, 2, "multi-file 1-cfa call-site count");
  assertEqual(
    callSiteSummary.zeroCfa.entries[0]?.contextKey,
    "expression-domain-class-value@<root>",
    "zero-cfa root context key",
  );
  assertEqual(
    callSiteSummary.zeroCfa.entries[1]?.contextKey,
    "expression-domain-class-value@<root>",
    "second zero-cfa root context key",
  );
  assertEqual(
    callSiteSummary.oneCfa.entries[0]?.contextKey,
    "expression-domain-class-value@/tmp/App.tsx:expression-domain-flow",
    "one-cfa graph context key",
  );
  assertEqual(
    callSiteSummary.oneCfa.entries[1]?.contextKey,
    "expression-domain-class-value@/tmp/Card.tsx:expression-domain-flow",
    "second one-cfa graph context key",
  );

  process.stdout.write(
    `validated expression-domain flow analysis: graphs=${summary.analyses.length} nodes=${analysis?.nodes.length ?? 0} callSiteProduct=${callSiteSummary.product}\n`,
  );
})();

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
