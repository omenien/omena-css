import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const cargoArguments = [
  "test",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-abstract-value",
  "cfa",
] as const;
const requiredTests = [
  "analyzes_k_cfa_limited_call_site_flows_with_context_stack_discrimination",
  "analyzes_one_cfa_call_site_flows_with_context_discrimination",
  "analyzes_one_cfa_class_value_flow_with_branch_merge_and_refinement",
  "cfa_context_sensitivity_labels_match_supplied_inputs",
  "distinguishes_zero_cfa_and_one_cfa_call_site_abstractions",
] as const;
const contextLabelDispositions = [
  ["summarize_omena_abstract_value_flow_analysis", "perSuppliedGraph", "changed"],
  ["analyze_class_value_flow", "perSuppliedGraph", "changed"],
  ["analyze_class_value_control_flow_graph", "perSuppliedGraph", "changed"],
  ["analyze_one_cfa_call_site_flows", "1-cfa", "deliberately unchanged"],
  ["analyze_k_limited_call_site_flows", "{max_context_depth}-cfa", "deliberately unchanged"],
] as const;

const performanceDocumentation = readFileSync("docs/performance.md", "utf8");
for (const [producer, literal, disposition] of contextLabelDispositions) {
  const row = performanceDocumentation
    .split(/\r?\n/u)
    .find((line) => line.includes(`\`${producer}\``));
  assert.ok(
    row?.includes(`\`${literal}\``) && row.includes(`| ${disposition}`),
    `abstract-value context documentation is missing the disposition for ${producer}`,
  );
}

const flowSource = readFileSync("rust/crates/omena-abstract-value/src/flow.rs", "utf8");
for (const disclosure of [
  "Runs bounded deterministic iteration over one supplied graph without\n/// deriving a call graph or call-site context.",
  "Partitions caller-supplied graphs by caller-supplied call-site identifiers.",
  "This does not derive a call graph or run an interprocedural fixed point.",
]) {
  assert.ok(flowSource.includes(disclosure), `abstract-value flow disclosure lost ${disclosure}`);
}

const expressionFlowCheck = readFileSync(
  "scripts/check-rust-expression-domain-flow-analysis.ts",
  "utf8",
);
assert.ok(
  expressionFlowCheck.includes('entry.analysis.contextSensitivity === "perSuppliedGraph"'),
  "expression-domain flow check must expect the supplied-graph wire label",
);

const engineInputProducerTests = readFileSync(
  "rust/crates/omena-engine-input-producers/src/expression_domain.rs",
  "utf8",
);
assert.match(
  engineInputProducerTests,
  /summary\.analyses\[0\]\.analysis\.context_sensitivity,\s+"perSuppliedGraph"/u,
  "engine-input-producers flow contract must expect the supplied-graph wire label",
);

const queryExpressionTests = readFileSync(
  "rust/crates/omena-query/src/tests/expression_domain.rs",
  "utf8",
);
for (const literal of [
  'summary.zero_cfa.context_sensitivity, "0-cfa"',
  'summary.one_cfa.context_sensitivity, "1-cfa"',
]) {
  assert.ok(
    queryExpressionTests.includes(literal),
    `query call-site context compatibility assertion lost ${literal}`,
  );
}

const deepeningCheck = readFileSync("scripts/check-rust-m8-dynamic-classname-deepening.ts", "utf8");
for (const literal of [
  'assert.equal(zeroCfa.contextSensitivity, "0-cfa")',
  'assert.equal(twoCfa.contextSensitivity, "2-cfa")',
]) {
  assert.ok(
    deepeningCheck.includes(literal),
    `dynamic-classname call-site context assertion lost ${literal}`,
  );
}

const sourceDynamicTests = readFileSync(
  "rust/crates/omena-lsp-server/src/tests/source_dynamic.rs",
  "utf8",
);
assert.ok(
  sourceDynamicTests.includes(
    "backed by flow analysis over the supplied graph, not a local source scan",
  ),
  "LSP dynamic-source provenance must disclose supplied-graph flow",
);
assert.ok(
  !sourceDynamicTests.includes("backed by the query/checker k-CFA flow"),
  "LSP dynamic-source provenance must not claim call-graph-derived k-CFA",
);

const listedTests = execFileSync("cargo", [...cargoArguments, "--", "--list"], {
  cwd: process.cwd(),
  encoding: "utf8",
});
for (const testName of requiredTests) {
  assert.match(
    listedTests,
    new RegExp(`^tests::${testName}: test$`, "mu"),
    `abstract-value context gate did not discover ${testName}`,
  );
}
const discoveredTests = [...listedTests.matchAll(/^tests::([^:]+): test$/gmu)].map(
  (match) => match[1],
);
assert.deepEqual(
  discoveredTests.toSorted(),
  [...requiredTests].toSorted(),
  "abstract-value context gate required-test set drifted from the discovered contract suite",
);

execFileSync("cargo", cargoArguments, {
  cwd: process.cwd(),
  stdio: "inherit",
});

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-abstract-value.context-label-contract",
      discoveredContractTests: requiredTests.length,
      contextLabelDispositionRows: contextLabelDispositions.length,
    },
    null,
    2,
  ),
);
