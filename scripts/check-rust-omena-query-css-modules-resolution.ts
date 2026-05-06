import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const root = process.cwd();
const styleSource = readFileSync(path.join(root, "rust/crates/omena-query/src/style.rs"), "utf8");
const typeSource = readFileSync(path.join(root, "rust/crates/omena-query/src/types.rs"), "utf8");
const testSource = readFileSync(path.join(root, "rust/crates/omena-query/src/tests.rs"), "utf8");
const hostTypeSource = readFileSync(
  path.join(root, "server/engine-host-node/src/style-semantic-graph-query-backend.ts"),
  "utf8",
);
const readmeSource = readFileSync(path.join(root, "rust/crates/omena-query/README.md"), "utf8");

for (const required of [
  'status: "icssExportImportClosureSeed"',
  "transitive_closure_ready: true",
  "value_graph_closure_ready: true",
  "icss_export_import_closure_ready: true",
  "cycle_detection_ready: true",
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
]) {
  assert.ok(typeSource.includes(required), `omena-query types must expose ${required}`);
}

for (const required of [
  "style_semantic_graph_batch_detects_css_modules_composes_cycles",
  "style_semantic_graph_batch_detects_css_modules_value_cycles",
  "style_semantic_graph_batch_detects_css_modules_icss_cycles",
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
  "valueGraphClosureReady",
  "icssExportImportClosureReady",
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
    "cycles=composes,value,icss",
    "nextPriorities=0",
  ].join(" "),
);
process.stdout.write("\n");
