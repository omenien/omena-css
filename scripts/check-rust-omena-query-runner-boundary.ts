import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const RUNNER_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/src/main.rs");

const OMENA_QUERY_OWNED_COMMANDS = new Map([
  ["input-omena-query-boundary", "summarize_omena_query_boundary"],
  [
    "omena-query-selected-query-adapter-capabilities",
    "summarize_omena_query_selected_query_adapter_capabilities",
  ],
  [
    "input-source-resolution-query-fragments",
    "summarize_omena_query_source_resolution_query_fragments",
  ],
  [
    "input-expression-semantics-query-fragments",
    "summarize_omena_query_expression_semantics_query_fragments",
  ],
  ["input-selector-usage-query-fragments", "summarize_omena_query_selector_usage_query_fragments"],
  [
    "input-source-resolution-canonical-producer",
    "summarize_omena_query_source_resolution_canonical_producer_signal",
  ],
  [
    "input-omena-resolver-source-resolution-runtime",
    "summarize_omena_query_source_resolution_runtime",
  ],
  [
    "input-expression-semantics-canonical-producer",
    "summarize_omena_query_expression_semantics_canonical_producer_signal",
  ],
  [
    "input-expression-domain-flow-analysis",
    "summarize_omena_query_expression_domain_flow_analysis",
  ],
  [
    "input-expression-domain-control-flow-analysis",
    "summarize_omena_query_expression_domain_control_flow_analysis",
  ],
  [
    "input-expression-domain-incremental-flow-analysis",
    "summarize_omena_query_expression_domain_incremental_flow_analysis",
  ],
  [
    "input-expression-domain-selector-projection",
    "summarize_omena_query_expression_domain_selector_projection",
  ],
  [
    "input-selector-usage-canonical-producer",
    "summarize_omena_query_selector_usage_canonical_producer_signal",
  ],
  ["omena-parser-style-facts", "summarize_omena_query_omena_parser_style_facts"],
  ["style-semantic-graph", "summarize_omena_query_style_semantic_graph_from_source"],
  ["read-cascade-at-position", "read_omena_query_cascade_at_position"],
  ["style-semantic-graph-batch", "summarize_omena_query_style_semantic_graph_batch_from_sources"],
  ["transform-plan", "summarize_omena_query_transform_plan_from_source"],
  ["transform-execute", "execute_omena_query_transform_passes_from_source"],
] as const);

const DIRECT_PRODUCER_LANE_COMMANDS = new Map([
  ["input-type-facts", "summarize_type_fact_input"],
  ["input-query-plan", "summarize_query_plan_input"],
  ["input-expression-domains", "summarize_expression_domain_plan_input"],
  ["input-expression-domain-fragments", "summarize_expression_domain_fragments_input"],
  ["input-expression-domain-candidates", "summarize_expression_domain_candidates_input"],
  [
    "input-expression-domain-canonical-candidate",
    "summarize_expression_domain_canonical_candidate_bundle_input",
  ],
  [
    "input-expression-domain-evaluator-candidates",
    "summarize_expression_domain_evaluator_candidates_input",
  ],
  [
    "input-expression-domain-canonical-producer",
    "summarize_expression_domain_canonical_producer_signal_input",
  ],
  ["input-selector-usage-plan", "summarize_selector_usage_plan_input"],
  ["input-selector-usage-fragments", "summarize_selector_usage_fragments_input"],
  ["input-selector-usage-candidates", "summarize_selector_usage_candidates_input"],
  [
    "input-selector-usage-evaluator-candidates",
    "summarize_selector_usage_evaluator_candidates_input",
  ],
  [
    "input-selector-usage-canonical-candidate",
    "summarize_selector_usage_canonical_candidate_bundle_input",
  ],
  ["input-source-resolution-plan", "summarize_source_resolution_plan_input"],
  ["input-expression-semantics-fragments", "summarize_expression_semantics_fragments_input"],
  ["input-expression-semantics-candidates", "summarize_expression_semantics_candidates_input"],
  [
    "input-expression-semantics-evaluator-candidates",
    "summarize_expression_semantics_evaluator_candidates_input",
  ],
  [
    "input-expression-semantics-canonical-candidate",
    "summarize_expression_semantics_canonical_candidate_bundle_input",
  ],
  ["input-source-side-canonical-producer", "summarize_source_side_canonical_producer_signal_input"],
  [
    "input-source-side-canonical-candidate",
    "summarize_source_side_canonical_candidate_bundle_input",
  ],
  ["input-source-side-evaluator-candidates", "summarize_source_side_evaluator_candidates_input"],
  ["input-semantic-canonical-candidate", "summarize_semantic_canonical_candidate_bundle_input"],
  ["input-semantic-evaluator-candidates", "summarize_semantic_evaluator_candidates_input"],
  ["input-semantic-canonical-producer", "summarize_semantic_canonical_producer_signal_input"],
  [
    "input-expression-semantics-match-fragments",
    "summarize_expression_semantics_match_fragments_input",
  ],
  ["input-source-resolution-fragments", "summarize_source_resolution_fragments_input"],
  ["input-source-resolution-candidates", "summarize_source_resolution_candidates_input"],
  [
    "input-source-resolution-evaluator-candidates",
    "summarize_source_resolution_evaluator_candidates_input",
  ],
  [
    "input-source-resolution-canonical-candidate",
    "summarize_source_resolution_canonical_candidate_bundle_input",
  ],
  ["input-source-resolution-match-fragments", "summarize_source_resolution_match_fragments_input"],
] as const);

const runnerSource = readFileSync(RUNNER_PATH, "utf8");
const commandBodies = extractCommandBodies(runnerSource);

for (const [command, expectedCall] of OMENA_QUERY_OWNED_COMMANDS) {
  const body = commandBodies.get(command);
  assert.ok(body, `missing engine-shadow-runner command arm: ${command}`);
  assert.ok(body.includes(expectedCall), `command ${command} must route through ${expectedCall}`);
  assert.equal(
    findDirectProducerCalls(body).length,
    0,
    `command ${command} must not call engine-input-producers directly`,
  );
}

const actualDirectProducerCalls = [...commandBodies.entries()]
  .flatMap(([command, body]) =>
    findDirectProducerCalls(body).map((functionName) => [command, functionName] as const),
  )
  .toSorted(([leftCommand], [rightCommand]) => leftCommand.localeCompare(rightCommand));

assert.deepEqual(
  actualDirectProducerCalls,
  [...DIRECT_PRODUCER_LANE_COMMANDS.entries()].toSorted(([leftCommand], [rightCommand]) =>
    leftCommand.localeCompare(rightCommand),
  ),
  "direct engine-input-producers calls must remain limited to explicit lower-level runner lane commands",
);

process.stdout.write(
  [
    "validated omena-query runner boundary:",
    `omenaOwnedCommands=${OMENA_QUERY_OWNED_COMMANDS.size}`,
    `directProducerLaneCommands=${DIRECT_PRODUCER_LANE_COMMANDS.size}`,
  ].join(" "),
);
process.stdout.write("\n");

function extractCommandBodies(source: string): Map<string, string> {
  const commandMatches = [...source.matchAll(/Some\("([^"]+)"\)\s*=>\s*\{/g)];
  const bodies = new Map<string, string>();

  for (const match of commandMatches) {
    const command = match[1];
    const bodyStart = match.index === undefined ? -1 : match.index + match[0].length;
    if (!command || bodyStart < 0) continue;
    bodies.set(command, readBraceBody(source, bodyStart));
  }

  return bodies;
}

function readBraceBody(source: string, bodyStart: number): string {
  let depth = 1;
  let index = bodyStart;
  while (index < source.length && depth > 0) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") depth -= 1;
    index += 1;
  }
  return source.slice(bodyStart, index - 1);
}

function findDirectProducerCalls(body: string): string[] {
  return [...body.matchAll(/\b(summarize_[A-Za-z0-9_]+_input)\s*\(/g)]
    .map((match) => match[1])
    .filter((functionName): functionName is string => functionName !== undefined);
}
