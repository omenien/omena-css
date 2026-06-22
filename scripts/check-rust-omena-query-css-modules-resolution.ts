import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const root = process.cwd();
const styleSource = readFileSync(path.join(root, "rust/crates/omena-query/src/style.rs"), "utf8");
const crossFileSummarySource = readFileSync(
  path.join(root, "rust/crates/omena-query/src/style/cross_file_summary.rs"),
  "utf8",
);
const crossFileSubstrateSource = readFileSync(
  path.join(root, "rust/crates/omena-cross-file-summary/src/lib.rs"),
  "utf8",
);
const crossFileHypergraphSource = [
  readFileSync(
    path.join(root, "rust/crates/omena-query/src/style/cross_file_hypergraph/mod.rs"),
    "utf8",
  ),
  crossFileSubstrateSource,
].join("\n");
const typeSource = [
  readFileSync(path.join(root, "rust/crates/omena-query/src/types.rs"), "utf8"),
  crossFileSubstrateSource,
].join("\n");
const testSource = [
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests.rs"), "utf8"),
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests/cross_file_summary.rs"), "utf8"),
  readFileSync(path.join(root, "rust/crates/omena-query/src/tests/style_diagnostics.rs"), "utf8"),
  readFileSync(
    path.join(root, "rust/crates/omena-query/src/tests/style_semantic_graph.rs"),
    "utf8",
  ),
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
  "OmenaQueryCrossFileSccEvidenceV0",
  "cross_file_scc",
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
  "summarize_omena_query_unified_cross_file_scc_report",
  "OmenaQueryUnifiedCrossFileSccReportV0",
  "exactTarjanScc",
  "notClaimedExactTraversal",
  "fixtureWitnessExactTarjanScc",
  "theorem_claimed: false",
  "canonical_scc_node_id",
]) {
  assert.ok(
    crossFileHypergraphSource.includes(required),
    `omena-query cross-file hypergraph must retain ${required}`,
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
  "cross_file_hypergraph_reports_exact_tarjan_scc_for_composes_cycle",
  "style_diagnostics_for_workspace_file_flags_unified_cross_file_composes_cycle",
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
  "layerMarker",
  "featureGate",
  "semiringIdentifier",
]) {
  assert.ok(hostTypeSource.includes(required), `engine host type surface must expose ${required}`);
}

assert.match(
  readmeSource,
  /transitive closure, and\s+cycle detection/s,
  "omena-query README must describe CSS Modules closure/cycle support",
);

// OmenaQuerySassModuleCrossFileResolutionV0 derives Serialize and is emitted to the engine host
// (style-semantic-graph batch -> JSON stdout). Freeze its serialized field set so an accidental
// new sub-field that silently widens the public boundary fails CI rather than leaking.
const resolutionStructBody = /pub struct OmenaQuerySassModuleCrossFileResolutionV0 \{([\s\S]*?)\n\}/
  .exec(typeSource)?.[1];
assert.ok(
  resolutionStructBody,
  "OmenaQuerySassModuleCrossFileResolutionV0 must be defined in omena-query types",
);
const resolutionFields = [...resolutionStructBody.matchAll(/pub (\w+):/g)].map((match) => match[1]);
assert.deepEqual(
  resolutionFields,
  [
    "schema_version",
    "product",
    "status",
    "resolution_scope",
    "style_count",
    "module_edge_count",
    "resolved_module_edge_count",
    "unresolved_module_edge_count",
    "external_module_edge_count",
    "symlink_chain_edge_count",
    "symlink_chain_link_count",
    "configured_module_instance_count",
    "edges",
    "graph_closure_edge_count",
    "cycle_count",
    "visibility_filter_count",
    "graph_closure_edges",
    "cycles",
    "capabilities",
    "next_priorities",
  ],
  "OmenaQuerySassModuleCrossFileResolutionV0 serialized field set changed — this type crosses the engine-host boundary; update this allowlist (and the host mirror) deliberately.",
);

process.stdout.write(
  [
    "validated omena-query CSS Modules resolution:",
    "crossFileResolutionFields=20",
    "closure=composes,value,icss",
    "cycles=composes,value,icss,sass",
    "scc=exact-tarjan",
    "sassClosure=moduleGraph",
    "summaryEdges=style,source",
    "nextPriorities=0",
  ].join(" "),
);
process.stdout.write("\n");
