import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const RUNNER_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/src/main.rs");
const QUERY_SRC_DIR = path.join(process.cwd(), "rust/crates/omena-query/src");
// The M8-2 query split relocated the source-resolution routing fragments into
// omena-query-core, so the query layer now spans both crates.
const QUERY_CORE_SRC_DIR = path.join(process.cwd(), "rust/crates/omena-query-core/src");

const runnerSource = readFileSync(RUNNER_PATH, "utf8");
const querySource = `${readRustSourceDirectory(QUERY_SRC_DIR)}\n${readRustSourceDirectory(
  QUERY_CORE_SRC_DIR,
)}`;
const commandBodies = extractCommandBodies(runnerSource);

const resolverBoundaryBody = commandBodies.get("input-omena-resolver-boundary");
assert.ok(
  resolverBoundaryBody,
  "missing engine-shadow-runner command arm: input-omena-resolver-boundary",
);
assert.ok(
  resolverBoundaryBody.includes("summarize_omena_resolver_boundary"),
  "input-omena-resolver-boundary must route through omena-resolver",
);

const resolverModuleGraphBody = commandBodies.get("input-omena-resolver-module-graph");
assert.ok(
  resolverModuleGraphBody,
  "missing engine-shadow-runner command arm: input-omena-resolver-module-graph",
);
assert.ok(
  resolverModuleGraphBody.includes("summarize_omena_resolver_module_graph_index"),
  "input-omena-resolver-module-graph must route through omena-resolver",
);

const resolverSourceResolutionRuntimeBody = commandBodies.get(
  "input-omena-resolver-source-resolution-runtime",
);
assert.ok(
  resolverSourceResolutionRuntimeBody,
  "missing engine-shadow-runner command arm: input-omena-resolver-source-resolution-runtime",
);
assert.ok(
  resolverSourceResolutionRuntimeBody.includes("summarize_omena_query_source_resolution_runtime"),
  "input-omena-resolver-source-resolution-runtime must route through the omena-query wrapper",
);

const resolverRuntimeQueryBody = commandBodies.get("omena-resolver-runtime-query-boundary");
assert.ok(
  resolverRuntimeQueryBody,
  "missing engine-shadow-runner command arm: omena-resolver-runtime-query-boundary",
);
assert.ok(
  resolverRuntimeQueryBody.includes("OmenaResolverModuleGraphSummaryV0"),
  "omena-resolver-runtime-query-boundary must deserialize the resolver module graph product",
);
assert.ok(
  resolverRuntimeQueryBody.includes("summarize_omena_resolver_runtime_query_boundary"),
  "omena-resolver-runtime-query-boundary must route through omena-resolver",
);

const resolverStyleModuleResolutionBody = commandBodies.get(
  "omena-resolver-style-module-resolution",
);
assert.ok(
  resolverStyleModuleResolutionBody,
  "missing engine-shadow-runner command arm: omena-resolver-style-module-resolution",
);
assert.ok(
  resolverStyleModuleResolutionBody.includes("OmenaResolverStyleModuleResolutionInputV0"),
  "omena-resolver-style-module-resolution must deserialize the resolver style-module input product",
);
assert.ok(
  resolverStyleModuleResolutionBody.includes("OmenaResolverStylePackageManifestV0"),
  "omena-resolver-style-module-resolution must map package manifests into omena-resolver contracts",
);
assert.ok(
  resolverStyleModuleResolutionBody.includes("OmenaResolverTsconfigPathMappingV0"),
  "omena-resolver-style-module-resolution must map tsconfig path aliases into omena-resolver contracts",
);
assert.ok(
  resolverStyleModuleResolutionBody.includes("OmenaResolverBundlerPathAliasMappingV0"),
  "omena-resolver-style-module-resolution must map bundler path aliases into omena-resolver contracts",
);
assert.ok(
  resolverStyleModuleResolutionBody.includes(
    "summarize_omena_resolver_style_module_resolution_with_path_mappings",
  ),
  "omena-resolver-style-module-resolution must route through omena-resolver",
);
assert.ok(
  runnerSource.includes('"omena-resolver-style-module-resolution" =>'),
  "engine-shadow-runner daemon must support omena-resolver-style-module-resolution",
);

const resolverSpecifierRuntimeBody = commandBodies.get(
  "omena-resolver-specifier-resolution-runtime",
);
assert.ok(
  resolverSpecifierRuntimeBody,
  "missing engine-shadow-runner command arm: omena-resolver-specifier-resolution-runtime",
);
assert.ok(
  resolverSpecifierRuntimeBody.includes("OmenaResolverSpecifierResolutionRuntimeInputV0"),
  "omena-resolver-specifier-resolution-runtime must deserialize the resolver specifier runtime input product",
);
assert.ok(
  resolverSpecifierRuntimeBody.includes("OmenaResolverBundlerPathAliasMappingV0"),
  "omena-resolver-specifier-resolution-runtime must map bundler path aliases into omena-resolver contracts",
);
assert.ok(
  resolverSpecifierRuntimeBody.includes(
    "summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings",
  ),
  "omena-resolver-specifier-resolution-runtime must route through omena-resolver",
);
assert.ok(
  runnerSource.includes('"omena-resolver-specifier-resolution-runtime" =>'),
  "engine-shadow-runner daemon must support omena-resolver-specifier-resolution-runtime",
);

assert.ok(
  querySource.includes("summarize_omena_resolver_query_fragments(input)"),
  "omena-query source-resolution query fragments must route through omena-resolver",
);
assert.ok(
  querySource.includes("summarize_omena_resolver_canonical_producer_signal(input)"),
  "omena-query source-resolution canonical producer must route through omena-resolver",
);
assert.ok(
  querySource.includes("summarize_omena_resolver_source_resolution_runtime(input)"),
  "omena-query source-resolution runtime index must route through omena-resolver",
);

process.stdout.write(
  [
    "validated omena-resolver runner boundary:",
    "resolverBoundaryCommand=input-omena-resolver-boundary",
    "moduleGraphCommand=input-omena-resolver-module-graph",
    "sourceResolutionRuntimeCommand=input-omena-resolver-source-resolution-runtime",
    "runtimeQueryCommand=omena-resolver-runtime-query-boundary",
    "styleModuleResolutionCommand=omena-resolver-style-module-resolution",
    "specifierResolutionRuntimeCommand=omena-resolver-specifier-resolution-runtime",
    "bundlerPathAliasMapping=runner-input",
    "tsconfigPathMapping=runner-input",
    "queryDelegation=source-resolution",
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

function readRustSourceDirectory(directory: string): string {
  return readdirSync(directory)
    .filter((entry) => entry.endsWith(".rs"))
    .toSorted()
    .map((entry) => readFileSync(path.join(directory, entry), "utf8"))
    .join("\n");
}
