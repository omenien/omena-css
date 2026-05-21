import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const root = process.cwd();
const styleSource = readFileSync(path.join(root, "rust/crates/omena-query/src/style.rs"), "utf8");
const crossFileSummarySource = readFileSync(
  path.join(root, "rust/crates/omena-query/src/style/cross_file_summary.rs"),
  "utf8",
);
const typeSource = readFileSync(path.join(root, "rust/crates/omena-query/src/types.rs"), "utf8");
const testSource = [
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests.rs"), "utf8"),
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests/cross_file_summary.rs"), "utf8"),
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests/style_semantic_graph.rs"), "utf8"),
].join("\n");
const hostTypeSource = readFileSync(
  path.join(root, "server/engine-host-node/src/style-semantic-graph-query-backend.ts"),
  "utf8",
);
const readmeSource = readFileSync(path.join(root, "rust/crates/omena-query/README.md"), "utf8");

for (const required of [
  'status: "icssExportImportClosureSeed"',
  'status: "moduleGraphClosureResolved"',
  "transitive_closure_ready: true",
  "value_graph_closure_ready: true",
  "icss_export_import_closure_ready: true",
  "cycle_detection_ready: true",
  "graph_closure_ready: true",
  "namespace_show_hide_filter_ready: true",
  "next_priorities: vec![]",
]) {
  assert.ok(styleSource.includes(required), `omena-query style resolver must retain ${required}`);
}

for (const required of [
  "composes_closure_edges",
  "value_closure_edges",
  "icss_closure_edges",
  "composes_cycle_count",
  "value_cycle_count",
  "icss_cycle_count",
  "OmenaQueryCssModulesComposesClosureEdgeV0",
  "OmenaQueryCssModulesValueClosureEdgeV0",
  "OmenaQueryCssModulesIcssClosureEdgeV0",
  "OmenaQuerySassModuleGraphClosureEdgeV0",
  "OmenaQueryCrossFileSummaryV0",
  "OmenaQueryCrossFileSummaryEdgeKindCountV0",
  "OmenaQueryCrossFileSummaryEdgeV0",
  "edge_kind_counts",
  "from_path",
  "target_path",
  "linear_provenance",
  "visibility_filter_names",
  "namespace_show_hide_filter_ready",
]) {
  assert.ok(typeSource.includes(required), `omena-query types must expose ${required}`);
}

for (const required of [
  'product: "omena-query.cross-file-summary"',
  'status: "summaryEdgeSeed"',
  'status: "sourceSelectorSummaryEdgeSeed"',
  "stable_omena_query_cross_file_summary_hash",
  "cssModulesComposesClosure",
  "cssModulesValueClosure",
  "cssModulesIcssClosure",
  "sassModuleGraphClosure",
  "styleDesignTokenReference",
  "sourceSelectorReference",
  "source_selector_reference_edges_ready: true",
  "summarize_omena_query_workspace_cross_file_summary",
  '"workspaceSummaryEdgeSeed"',
  '"workspaceStyleAndSource"',
  "workspaceSummaryHashInvalidationGate",
  "summarize_omena_query_cross_file_summary_edge_kind_counts",
  "edge.linear_provenance.semiring_identifier()",
  "edge.linear_provenance.terms",
]) {
  assert.ok(
    crossFileSummarySource.includes(required),
    `omena-query cross-file summary must retain ${required}`,
  );
}

for (const required of [
  "style_semantic_graph_batch_detects_css_modules_composes_cycles",
  "style_semantic_graph_batch_detects_css_modules_value_cycles",
  "style_semantic_graph_batch_detects_css_modules_icss_cycles",
  "style_semantic_graph_batch_resolves_sass_module_graph_closure_and_filters",
  "style_semantic_graph_batch_detects_sass_module_cycles",
  "style_semantic_graph_batch_cross_file_summary_hash_tracks_edge_changes",
  "cross_file_summary_linear_provenance_serializes_as_strict_superset",
  "cross_file_summary_edges_are_equivalent_to_resolution_products",
  "source_selector_references_emit_cross_file_summary_edges",
  "workspace_cross_file_summary_merges_style_and_source_edge_sets",
  "workspace_cross_file_summary_linear_provenance_covers_merged_style_and_source_edges",
  "workspace_cross_file_summary_hash_tracks_source_selector_changes",
  "workspace_cross_file_summary_hash_tracks_style_edge_changes",
  "workspace_cross_file_summary_hash_is_input_order_stable",
  "workspace_cross_file_summary_hash_tracks_package_manifest_changes",
  "workspace_cross_file_summary_reports_edge_kind_counts_for_m4_vocabulary",
  "transitive_composes",
  "transitive_value",
  "transitive_icss",
]) {
  assert.ok(testSource.includes(required), `omena-query tests must cover ${required}`);
}

for (const required of [
  "composesClosureEdges",
  "valueClosureEdges",
  "icssClosureEdges",
  "graphClosureEdges",
  "visibilityFilterNames",
  "namespaceShowHideFilterReady",
  "valueGraphClosureReady",
  "icssExportImportClosureReady",
  "crossFileSummary?: StyleSemanticGraphCrossFileSummaryV0",
  "StyleSemanticGraphCrossFileSummaryEdgeV0",
  "StyleSemanticGraphCrossFileSummaryEdgeKindCountV0",
  "edgeKindCounts",
  "fromPath",
  "targetPath",
  "linearProvenance",
  "semiringIdentifier",
]) {
  assert.ok(hostTypeSource.includes(required), `engine host type surface must expose ${required}`);
}

assert.match(
  readmeSource,
  /transitive closure, and\s+cycle detection/s,
  "omena-query README must describe CSS Modules closure/cycle support",
);

process.stdout.write(
  [
    "validated omena-query CSS Modules resolution:",
    "closure=composes,value,icss",
    "cycles=composes,value,icss,sass",
    "sassClosure=moduleGraph",
    "summaryEdges=style,source",
    "nextPriorities=0",
  ].join(" "),
);
process.stdout.write("\n");
